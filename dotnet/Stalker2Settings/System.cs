using System.Management;
using Stalker2Settings.Models;

namespace Stalker2Settings;

/// <summary>
/// Handles system information retrieval and analysis
/// </summary>
public static class System
{
    /// <summary>
    /// Get system hardware information
    /// </summary>
    public static SystemInfo GetSystemInfo()
    {
        var systemInfo = new SystemInfo();

        try
        {
            // Get CPU information
            using (var searcher = new ManagementObjectSearcher("SELECT Name, NumberOfCores FROM Win32_Processor"))
            {
                foreach (var obj in searcher.Get())
                {
                    systemInfo.CpuName = obj["Name"]?.ToString() ?? "Unknown CPU";
                    systemInfo.CpuCores = Convert.ToUInt32(obj["NumberOfCores"]);
                    break; // Just take the first processor
                }
            }

            // Get RAM information
            using (var searcher = new ManagementObjectSearcher("SELECT TotalPhysicalMemory FROM Win32_ComputerSystem"))
            {
                foreach (var obj in searcher.Get())
                {
                    var totalBytes = Convert.ToUInt64(obj["TotalPhysicalMemory"]);
                    systemInfo.RamGb = totalBytes / 1024 / 1024 / 1024;
                    break;
                }
            }

            // Get GPU information
            using (var searcher = new ManagementObjectSearcher("SELECT Name, AdapterRAM FROM Win32_VideoController"))
            {
                foreach (var obj in searcher.Get())
                {
                    systemInfo.GpuName = obj["Name"]?.ToString() ?? "Unknown GPU";
                    try
                    {
                        var adapterRam = Convert.ToUInt64(obj["AdapterRAM"]);
                        systemInfo.GpuVramMb = adapterRam / 1024 / 1024;
                    }
                    catch
                    {
                        // Some GPUs might not report VRAM correctly
                        systemInfo.GpuVramMb = EstimateVRam(systemInfo.GpuName);
                    }
                    break; // Just take the first GPU
                }
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error getting system info: {ex.Message}");
            // Set fallback values
            if (string.IsNullOrEmpty(systemInfo.CpuName)) systemInfo.CpuName = "Unknown CPU";
            if (systemInfo.CpuCores == 0) systemInfo.CpuCores = 4;
            if (systemInfo.RamGb == 0) systemInfo.RamGb = 8;
            if (string.IsNullOrEmpty(systemInfo.GpuName)) systemInfo.GpuName = "Unknown GPU";
            if (systemInfo.GpuVramMb == 0) systemInfo.GpuVramMb = 4096;
        }

        return systemInfo;
    }

    /// <summary>
    /// Estimate VRAM based on GPU name if not reported correctly
    /// </summary>
    private static ulong EstimateVRam(string gpuName)
    {
        gpuName = gpuName.ToLower();

        // NVIDIA GPUs
        if (gpuName.Contains("rtx 4090")) return 24576;
        if (gpuName.Contains("rtx 4080")) return 16384;
        if (gpuName.Contains("rtx 4070 ti")) return 12288;
        if (gpuName.Contains("rtx 4070")) return 12288;
        if (gpuName.Contains("rtx 4060 ti")) return 8192;
        if (gpuName.Contains("rtx 4060")) return 8192;
        if (gpuName.Contains("rtx 3090")) return 24576;
        if (gpuName.Contains("rtx 3080 ti")) return 12288;
        if (gpuName.Contains("rtx 3080")) return 10240;
        if (gpuName.Contains("rtx 3070 ti")) return 8192;
        if (gpuName.Contains("rtx 3070")) return 8192;
        if (gpuName.Contains("rtx 3060 ti")) return 8192;
        if (gpuName.Contains("rtx 3060")) return 12288;
        if (gpuName.Contains("rtx 2080 ti")) return 11264;
        if (gpuName.Contains("rtx 2080")) return 8192;
        if (gpuName.Contains("rtx 2070")) return 8192;
        if (gpuName.Contains("rtx 2060")) return 6144;
        if (gpuName.Contains("gtx 1080 ti")) return 11264;
        if (gpuName.Contains("gtx 1080")) return 8192;
        if (gpuName.Contains("gtx 1070 ti")) return 8192;
        if (gpuName.Contains("gtx 1070")) return 8192;
        if (gpuName.Contains("gtx 1060")) return 6144;
        if (gpuName.Contains("gtx 1050 ti")) return 4096;
        if (gpuName.Contains("gtx 1050")) return 2048;

        // AMD GPUs
        if (gpuName.Contains("rx 7900 xtx")) return 24576;
        if (gpuName.Contains("rx 7900 xt")) return 20480;
        if (gpuName.Contains("rx 7800 xt")) return 16384;
        if (gpuName.Contains("rx 7700 xt")) return 12288;
        if (gpuName.Contains("rx 7600")) return 8192;
        if (gpuName.Contains("rx 6950 xt")) return 16384;
        if (gpuName.Contains("rx 6900 xt")) return 16384;
        if (gpuName.Contains("rx 6800 xt")) return 16384;
        if (gpuName.Contains("rx 6800")) return 16384;
        if (gpuName.Contains("rx 6750 xt")) return 12288;
        if (gpuName.Contains("rx 6700 xt")) return 12288;
        if (gpuName.Contains("rx 6700")) return 10240;
        if (gpuName.Contains("rx 6650 xt")) return 8192;
        if (gpuName.Contains("rx 6600 xt")) return 8192;
        if (gpuName.Contains("rx 6600")) return 8192;
        if (gpuName.Contains("rx 6500 xt")) return 4096;
        if (gpuName.Contains("rx 5700 xt")) return 8192;
        if (gpuName.Contains("rx 5700")) return 8192;
        if (gpuName.Contains("rx 5600 xt")) return 6144;
        if (gpuName.Contains("rx 5500 xt")) return 8192;
        if (gpuName.Contains("rx 590")) return 8192;
        if (gpuName.Contains("rx 580")) return 8192;
        if (gpuName.Contains("rx 570")) return 8192;
        if (gpuName.Contains("rx 560")) return 4096;
        if (gpuName.Contains("rx 550")) return 4096;

        // Default fallback
        return 4096;
    }

    /// <summary>
    /// Get performance tier based on system info
    /// </summary>
    public static string GetPerformanceTier(SystemInfo systemInfo)
    {
        // Calculate a score based on CPU, RAM, and GPU
        int score = 0;

        // CPU score
        if (systemInfo.CpuCores >= 16) score += 30;
        else if (systemInfo.CpuCores >= 12) score += 25;
        else if (systemInfo.CpuCores >= 8) score += 20;
        else if (systemInfo.CpuCores >= 6) score += 15;
        else if (systemInfo.CpuCores >= 4) score += 10;
        else score += 5;

        // RAM score
        if (systemInfo.RamGb >= 64) score += 20;
        else if (systemInfo.RamGb >= 32) score += 15;
        else if (systemInfo.RamGb >= 16) score += 10;
        else if (systemInfo.RamGb >= 8) score += 5;
        else score += 0;

        // GPU score (based on VRAM)
        if (systemInfo.GpuVramMb >= 16384) score += 50;
        else if (systemInfo.GpuVramMb >= 12288) score += 40;
        else if (systemInfo.GpuVramMb >= 8192) score += 30;
        else if (systemInfo.GpuVramMb >= 6144) score += 20;
        else if (systemInfo.GpuVramMb >= 4096) score += 10;
        else score += 0;

        // Determine tier based on score
        if (score >= 80) return "Ultra";
        else if (score >= 60) return "High";
        else if (score >= 40) return "Medium";
        else if (score >= 20) return "Low";
        else return "Minimum";
    }
}