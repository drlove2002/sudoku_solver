from nicegui import ui

class SudokuGrid:
    def __init__(self, grid_size=9, on_change=None):
        self.grid_size = grid_size
        self.inputs = []
        self.on_change = on_change
        self.setup_ui()

    def setup_ui(self):
        with ui.grid(columns=9).classes('gap-1 mb-4'):
            for i in range(self.grid_size * self.grid_size):
                classes = 'w-10 h-10 text-center'
                r, c = i // 9, i % 9
                if c % 3 == 2 and c != 8:
                    classes += ' mr-1'
                if r % 3 == 2 and r != 8:
                    classes += ' mb-1'
                    
                inp = ui.input().classes(classes).props('dense outlined')
                if self.on_change:
                    inp.on('change', self.on_change)
                self.inputs.append(inp)

    def get_values(self):
        values = []
        for inp in self.inputs:
            val = inp.value
            if val and val.isdigit():
                values.append(int(val))
            else:
                values.append(0)
        return values

    def set_values(self, values):
        for i, val in enumerate(values):
            self.inputs[i].value = str(val) if val != 0 else ''
