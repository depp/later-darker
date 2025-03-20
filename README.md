# Later and Darker

Later and Darker is a demoscene experiment being made by Dietrich Epp in early 2025. Let's see how it turns out, shall we?

Later and Darker is licensed under the terms of the Mozilla Public License Version 2.0. See [LICENSE.txt](LICENSE.txt) for details.

## Prerequisites

- CMake
- vcpkg
- Rust

## Build

To build with Visual Studio, open the project folder with Visual Studio.

In Visual Studio, run **Project** → **Configure later-darker**. This will run the CMake configuration step and fetch the vcpkg dependencies. You can now select a target and run.

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

## Configuration

Intellisense VCR001 warnings may appear in Visual Studio.

https://developercommunity.visualstudio.com/t/Warning-VCR001---Function-definition-for/10702254

Go to **Tools** → **Options**.

Go to **Text Editor** → **C/C++** → **Intellisense**.

Change **Suggest create declaration/definition suggestion level** to **Refactoring only**.
