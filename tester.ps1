# 기존에 남아있을지 모르는 QEMU 프로세스 종료
stop-process -name qemu-system-x86_64 -ErrorAction SilentlyContinue

# 경로 설정
$QemuPath = "C:\Program Files\qemu"
$BuildTarget = "target\x86_64-unknown-uefi\debug\useless_efi.efi"
$EspDir = "esp"
$BootDir = "$EspDir\EFI\BOOT"
$BootFile = "$BootDir\BOOTX64.EFI"

# BIOS 파일 준비
$CodeFd = "$QemuPath\share\edk2-x86_64-code.fd"
$LocalVars = ".\ovmf_vars.fd"
if (-not (Test-Path $LocalVars)) {
    Copy-Item "$QemuPath\share\edk2-i386-vars.fd" $LocalVars
}

# ESP 디렉토리 및 부팅 파일 준비
if (-not (Test-Path $BootDir)) {
    New-Item -ItemType Directory -Force -Path $BootDir | Out-Null
}

Write-Host "Copying EFI executable to ESP..."
Copy-Item $BuildTarget $BootFile -Force

# message.txt 복사 (src 폴더에 있는 파일을 esp 루트로)
if (Test-Path "src\message.txt") {
    Write-Host "Copying message.txt from src to ESP Root..."
    Copy-Item "src\message.txt" "$EspDir\message.txt" -Force
}

# 실행 인자를 배열로 정의 (가장 안전한 방법)
$args = @(
    "-m", "512M",
    "-vga", "std",
    "-drive", "if=pflash,format=raw,readonly=on,file=$CodeFd",
    "-drive", "if=pflash,format=raw,file=$LocalVars",
    "-drive", "format=raw,file=fat:rw:$EspDir",
    "-net", "none"
)

Write-Host "Launching QEMU..."
# 배열 형태의 인자를 전달하여 실행
& "$QemuPath\qemu-system-x86_64.exe" @args