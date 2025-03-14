# Later and Darker

Later and Darker is a demoscene experiment being made by Dietrich Epp in early 2025. Let's see how it turns out, shall we?

Later and Darker is licensed under the terms of the Mozilla Public License Version 2.0. See [LICENSE.txt](LICENSE.txt) for details.

## Prerequisites

- CMake
- vcpkg
- Python
- Jinja2

To install the prerequisites on Windows:

```
winget install Python.Python.3.13
python3 -m pip install jinja2
```

## Debugging

In Visual Studio, go to **Debug** → **Debug and Launch Settings for later-darker**. This will bring up the `launch.vs.json` file. Add an `"args"` property to the configuration:

```json
{
  "version": "0.2.1",
  "defaults": {},
  "configurations": [
    {
      "type": "default",
      "project": "CMakeLists.txt",
      "projectTarget": "later-darker.exe",
      "name": "later-darker.exe",
      "args": [
        "AllocConsole=yes",
        "DebugContext=yes"
      ]
    }
  ]
}
```

## Clangd

Write the .clangd configuration:

    python config.py
