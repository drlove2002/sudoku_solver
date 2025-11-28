{
  description = "Python project with uv2nix - reusable template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    pyproject-nix.url = "github:pyproject-nix/pyproject.nix";
    pyproject-nix.inputs.nixpkgs.follows = "nixpkgs";

    uv2nix.url = "github:pyproject-nix/uv2nix";
    uv2nix.inputs.pyproject-nix.follows = "pyproject-nix";
    uv2nix.inputs.nixpkgs.follows = "nixpkgs";

    pyproject-build-systems.url = "github:pyproject-nix/build-system-pkgs";
    pyproject-build-systems.inputs.pyproject-nix.follows = "pyproject-nix";
    pyproject-build-systems.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      uv2nix,
      pyproject-nix,
      pyproject-build-systems,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        python = pkgs.python312;

        # Parse pyproject.toml at flake eval time
        pyprojectToml = builtins.fromTOML (builtins.readFile ./pyproject.toml);

        # Extract metadata automatically
        projectName = pyprojectToml.project.name;
        projectVersion = pyprojectToml.project.version;
        projectDescription = pyprojectToml.project.description;

        # Load workspace from uv.lock + pyproject.toml
        workspace = uv2nix.lib.workspace.loadWorkspace {
          workspaceRoot = ./.;
        };

        # Create overlay from uv.lock
        overlay = workspace.mkPyprojectOverlay {
          sourcePreference = "wheel";
        };

        # Build Python package set with all overlays
        pythonSet = pkgs.callPackage pyproject-nix.build.packages {
          inherit python;
        };

        # Apply overlays in correct order
        finalPythonSet = pythonSet.overrideScope (
          pkgs.lib.composeManyExtensions [
            pyproject-build-systems.overlays.default
            overlay
            (final: prev: {

            })
          ]
        );

        # Get your project package
        projectPackage = finalPythonSet.${projectName};

        # Create virtual environment with dependencies
        venv = finalPythonSet.mkVirtualEnv "${projectName}-env" workspace.deps.default;
        venvDev = finalPythonSet.mkVirtualEnv "${projectName}-dev-env" workspace.deps.all;

      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            venv
            pkgs.uv
            pkgs.ruff
          ];

          shellHook = ''
            echo "Entering ${projectName} dev environment"
            unset PYTHONPATH
            export UV_PYTHON="${python}/bin/python"
          '';
        };

        apps.default = {
          type = "app";
          program = "${venv}/bin/python";
          args = [
            "-m"
            projectName
          ];
        };

        apps.${projectName} = self.apps.${system}.default;
      }
    );
}
