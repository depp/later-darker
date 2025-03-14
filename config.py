import os
import platform
import subprocess
import sys
import yaml


def write_file(path, text):
    print("Writing", path, file=sys.stderr)
    with open(path, "w") as fp:
        fp.write(text)


def main():
    srcdir = os.path.dirname(os.path.abspath(__file__))
    system = platform.system()
    if system == "Darwin":
        preset = "macos-debug"
    else:
        print("Error: Unsupported system")
        raise SystemExit(1)
    print("Use preset:", preset, file=sys.stderr)
    data = {
        "CompileFlags": {
            "CompilationDatabase": os.path.join(
                srcdir, "out", "build", preset, "compile_commands.json"
            )
        }
    }
    text = yaml.dump(data, explicit_start=True)
    write_file(".clangd", text)


if __name__ == "__main__":
    main()
