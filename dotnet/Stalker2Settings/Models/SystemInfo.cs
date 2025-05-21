namespace Stalker2Settings.Models;

/// <summary>
/// Represents system hardware information
/// </summary>
public class SystemInfo
{
    public string CpuName { get; set; } = string.Empty;
    public uint CpuCores { get; set; }
    public ulong RamGb { get; set; }
    public string GpuName { get; set; } = string.Empty;
    public ulong GpuVramMb { get; set; }

    public SystemInfo()
    {
    }

    public SystemInfo(string cpuName, uint cpuCores, ulong ramGb, string gpuName, ulong gpuVramMb)
    {
        CpuName = cpuName;
        CpuCores = cpuCores;
        RamGb = ramGb;
        GpuName = gpuName;
        GpuVramMb = gpuVramMb;
    }

    public override string ToString()
    {
        return $"{CpuName} with {CpuCores} cores, {RamGb}GB RAM, {GpuName} ({GpuVramMb}MB VRAM)";
    }

    public SystemInfo Clone()
    {
        return new SystemInfo
        {
            CpuName = CpuName,
            CpuCores = CpuCores,
            RamGb = RamGb,
            GpuName = GpuName,
            GpuVramMb = GpuVramMb
        };
    }
}