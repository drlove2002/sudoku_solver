# Makefile for LaTeX documentation

DOCS = minigrid_relationship sudoku_solver_complete

.PHONY: all clean cleanall

all: $(addsuffix .pdf,$(DOCS))

%.pdf: %.tex
	pdflatex $<
	pdflatex $<  # Run twice for proper references

clean:
	rm -f *.aux *.log *.out *.toc *.lof *.lot *.fls *.fdb_latexmk *.synctex.gz

cleanall: clean
	rm -f $(addsuffix .pdf,$(DOCS))
