# Ejecuta las pruebas, compila los dos .exe y los dos .wasm, deja estos ultimos en web/ y
# prueba los cargadores. Requiere Rust con el target wasm32-unknown-unknown, y Node.
Set-Location $PSScriptRoot

function Ejecutar {
    param([string]$Programa, [string[]]$Argumentos)
    & $Programa @Argumentos
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Fallo: $Programa $($Argumentos -join ' ')"
        exit 1
    }
}

Ejecutar cargo @('test', '--workspace')
Ejecutar cargo @('build', '--release', '--workspace')
Ejecutar cargo @('build', '--release', '--workspace', '--lib', '--target', 'wasm32-unknown-unknown')
Copy-Item 'target\wasm32-unknown-unknown\release\mpp2xlsx.wasm', 'target\wasm32-unknown-unknown\release\doc2json.wasm' -Destination 'web\' -ErrorAction Stop
Ejecutar node @('web/prueba.mjs')
Write-Host 'Listo: ejecutables en target\release y modulos .wasm en web\.'
