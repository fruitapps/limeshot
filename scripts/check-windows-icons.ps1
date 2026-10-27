<#
.SYNOPSIS
  Verify both Windows executables embed the complete, current LimeShot icon.
.DESCRIPTION
  Maps resources as data (never executes the binaries), so x64 hosts can also check ARM64 builds.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string[]]$Executables,
    [string]$Icon = (Join-Path $PSScriptRoot '..\packaging\windows\LimeShot.ico')
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if (-not ('LimeShot.IconResources' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;

namespace LimeShot {
    public static class IconResources {
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        static extern IntPtr LoadLibraryEx(string name, IntPtr file, uint flags);
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        static extern IntPtr FindResource(IntPtr module, IntPtr name, IntPtr type);
        [DllImport("kernel32.dll", SetLastError = true)]
        static extern IntPtr LoadResource(IntPtr module, IntPtr resource);
        [DllImport("kernel32.dll")]
        static extern IntPtr LockResource(IntPtr resource);
        [DllImport("kernel32.dll", SetLastError = true)]
        static extern uint SizeofResource(IntPtr module, IntPtr resource);
        [DllImport("kernel32.dll")]
        static extern bool FreeLibrary(IntPtr module);

        static byte[] Read(IntPtr module, int type, int id) {
            IntPtr resource = FindResource(module, (IntPtr)id, (IntPtr)type);
            if (resource == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
            uint size = SizeofResource(module, resource);
            if (size == 0 || size > 16 * 1024 * 1024) throw new InvalidDataException("Invalid icon resource size");
            IntPtr loaded = LoadResource(module, resource);
            if (loaded == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
            IntPtr data = LockResource(loaded);
            if (data == IntPtr.Zero) throw new InvalidDataException("Cannot lock icon resource");
            byte[] bytes = new byte[(int)size];
            Marshal.Copy(data, bytes, 0, bytes.Length);
            return bytes;
        }

        public static void Verify(string exe, byte[] ico) {
            IntPtr module = LoadLibraryEx(exe, IntPtr.Zero, 2); // LOAD_LIBRARY_AS_DATAFILE
            if (module == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
            try {
                byte[] group = Read(module, 14, 1); // RT_GROUP_ICON, application icon ID
                if (ico.Length < 6 || group.Length < 6) throw new InvalidDataException("Missing icon header");
                for (int i = 0; i < 6; i++) {
                    if (ico[i] != group[i]) throw new InvalidDataException("Icon header/count mismatch");
                }
                int count = BitConverter.ToUInt16(ico, 4);
                if (count != 6 || group.Length != 6 + count * 14 || ico.Length < 6 + count * 16)
                    throw new InvalidDataException("Expected six icon sizes");
                for (int i = 0; i < count; i++) {
                    int entry = 6 + i * 16, embedded = 6 + i * 14;
                    for (int j = 0; j < 12; j++) {
                        if (ico[entry + j] != group[embedded + j])
                            throw new InvalidDataException("Icon directory mismatch");
                    }
                    uint size = BitConverter.ToUInt32(ico, entry + 8);
                    uint offset = BitConverter.ToUInt32(ico, entry + 12);
                    if ((ulong)offset + size > (ulong)ico.Length) throw new InvalidDataException("Truncated ICO");
                    byte[] image = Read(module, 3, BitConverter.ToUInt16(group, embedded + 12)); // RT_ICON
                    if (image.Length != size) throw new InvalidDataException("Icon image size mismatch");
                    for (int j = 0; j < image.Length; j++) {
                        if (image[j] != ico[(int)offset + j]) throw new InvalidDataException("Icon artwork mismatch");
                    }
                }
            } finally {
                FreeLibrary(module);
            }
        }
    }
}
'@
}
$artwork = [System.IO.File]::ReadAllBytes((Resolve-Path $Icon).Path)
foreach ($exe in $Executables) {
    $path = (Resolve-Path $exe).Path
    [LimeShot.IconResources]::Verify($path, $artwork)
    Write-Host "Verified six embedded LimeShot icon sizes: $path"
}
