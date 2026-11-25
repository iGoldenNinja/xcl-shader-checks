param(
    [Parameter(Mandatory = $true)][string]$DxcDirectory
)
$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$env:XCL_DXC_DIR = (Resolve-Path -LiteralPath $DxcDirectory).Path
$null = Get-Item -LiteralPath (Join-Path $env:XCL_DXC_DIR 'dxcompiler.dll')
$null = Get-Item -LiteralPath (Join-Path $env:XCL_DXC_DIR 'dxil.dll')
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_BUILD_JOBS = '2'
$run = New-Item -Type Directory (Join-Path $root ('run-' + [guid]::NewGuid().ToString('N')))
$env:CARGO_TARGET_DIR = Join-Path $run.FullName 'target'
$revisions = [ordered]@{
    recheck = '80f4819098e921f2e1ab371144d4832f68ffedd6'
    pr1948 = '1e4f2bdaddd438a31c778d43dc992c8a0455f6a9'
    pr1541 = 'a4f783459cecdac7a8e288e190b1fc78424bd9e5'
}
foreach ($name in $revisions.Keys) {
    $sha = $revisions[$name]
    $src = Join-Path $run.FullName "source-$name"
    & git init --quiet $src
    if ($LASTEXITCODE -ne 0) { throw "git init failed for $name" }
    & git -C $src fetch --quiet --depth 1 https://github.com/xcl-dev/xcl.git $sha
    if ($LASTEXITCODE -ne 0) { throw "could not fetch $sha" }
    & git -C $src checkout --quiet --detach FETCH_HEAD
    if ($LASTEXITCODE -ne 0 -or (& git -C $src rev-parse HEAD) -ne $sha) { throw "revision check failed" }
    $probe = (New-Item -Type Directory (Join-Path $run.FullName "probe-$name")).FullName
    $null = New-Item -Type Directory (Join-Path $probe 'src')
    Copy-Item -LiteralPath (Join-Path $root 'probe.rs') -Destination (Join-Path $probe 'src/main.rs')
    Copy-Item -LiteralPath (Join-Path $root "locks/$name.lock") -Destination (Join-Path $probe 'Cargo.lock')
    $dep = ($src + '/crates/xcl').Replace([char]92, [char]47)
    $toml = @('[package]', "name='xcl-check-$name'", "version='0.1.0'", "edition='2021'", '[dependencies]', "xcl={path='$dep'}")
    [IO.File]::WriteAllLines((Join-Path $probe 'Cargo.toml'), $toml)
    & cargo run --locked --manifest-path (Join-Path $probe 'Cargo.toml') -- (Join-Path $probe 'results') 2>&1 | Tee-Object (Join-Path $probe 'results.log')
    if ($LASTEXITCODE -ne 0) { throw "build, translation, validation or disassembly failed for $name" }
}
Write-Host "All results are in $($run.FullName)"
