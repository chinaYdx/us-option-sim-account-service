# DXC Build Notes

This is a DQTea DXC project. Do not edit generated files under `src/x_com/**`, generated `protos/import-api-*`, packaged `*.dxc`, or `target/**`.

After changing `protos/source-api-*`, run `adxbuilder init` before building.

Package this project to `D:\moomoo_targets` with:

```bat
package_to_moomoo_targets.bat
```

The wrapper runs `adxbuilder build` and then `adxbuilder pack`; pass `-SkipBuild` only when the DLL has already been built.

Use release mode with:

```bat
package_to_moomoo_targets.bat -Mode release
```

Output is written by project:

```text
D:\moomoo_targets\projects\us-option-sim-account-service\target
D:\moomoo_targets\projects\us-option-sim-account-service\package
D:\moomoo_targets\projects\us-option-sim-account-service\adxbuilder_debug.log
D:\moomoo_targets\projects\us-option-sim-account-service\adxbuilder_release.log
```

If an existing local `target` directory blocks packaging, rerun with `-MigrateLocalTarget` to move it into the project output directory and replace it with a junction.
