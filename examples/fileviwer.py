from dataclasses import dataclass
from functools import reduce
from os import DirEntry, scandir, stat
from pathlib import Path

from pyegui import *

HOME = Path.home()

@dataclass
class AppState:
    pwd: Path = HOME
    pwd_contents: [DirEntry] = None

    def set_pwd(self, entry):
        self.pwd = entry
        self.pwd_contents = None


APP_STATE = AppState()


# Source - https://stackoverflow.com/a/1094933
# Posted by Sridhar Ratnakumar, modified by community. See post 'Timeline' for change history
# Retrieved 2026-08-15, License - CC BY-SA 4.0
def sizeof_fmt(num, suffix="B"):
    for unit in ("", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi"):
        if abs(num) < 1024.0:
            return f"{num:3.1f}{unit}{suffix}"
        num /= 1024.0
    return f"{num:.1f}Yi{suffix}"

def files_view():
    # list files if needed
    if APP_STATE.pwd_contents is None:
        APP_STATE.pwd_contents = list(scandir(APP_STATE.pwd))

    with Layout(LayoutType.VerticalCenteredJustified):
        for entry in APP_STATE.pwd_contents:
            try:
                metadata = stat(entry)

                size_text = sizeof_fmt(metadata.st_size)
            except FileNotFoundError as e:
                size_text = "Unknown"
                print(e)

            entry_text = f"{entry.name}"

            with Layout(LayoutType.Horizontal):
                with Group():
                    if entry.is_dir():
                        if link_clicked(entry_text):
                            APP_STATE.set_pwd(Path(entry))
                    else:
                        label(entry_text)
                with Group():
                    label(size_text)

def control_panel():
    with Layout(LayoutType.Horizontal):
        parts = APP_STATE.pwd.parts

        for (i, part) in enumerate(parts):
            with Group():
                if link_clicked(part):
                    p = reduce(lambda acc, x: acc / x, parts[:i + 1], Path(parts[0]))
                    APP_STATE.set_pwd(p)

def main_view(ctx):
    control_panel()
    scroll_area_vertical(files_view)


if __name__ == "__main__":
    run_native("fileviwer", main_view)
