using System.Text;
using Stalker2Settings.Models;

namespace Stalker2Settings;

/// <summary>
/// Handles configuration file operations
/// </summary>
public static class Config
{
    /// <summary>
    /// Parse an INI file into a nested dictionary
    /// </summary>
    public static Dictionary<string, Dictionary<string, string>> ParseIniFile(string filePath)
    {
        var result = new Dictionary<string, Dictionary<string, string>>();
            
        if (!File.Exists(filePath))
            return result;

        string? currentSection = null;
        foreach (var line in File.ReadAllLines(filePath))
        {
            var trimmedLine = line.Trim();
                
            // Skip empty lines and comments
            if (string.IsNullOrWhiteSpace(trimmedLine) || trimmedLine.StartsWith(";"))
                continue;

            // Check if this is a section header
            if (trimmedLine.StartsWith("[") && trimmedLine.EndsWith("]"))
            {
                currentSection = trimmedLine.Substring(1, trimmedLine.Length - 2);
                if (!result.ContainsKey(currentSection))
                    result[currentSection] = new Dictionary<string, string>();
                continue;
            }

            // Skip if no section has been defined yet
            if (currentSection == null)
                continue;

            // Parse key-value pair
            var separatorIndex = trimmedLine.IndexOf('=');
            if (separatorIndex > 0)
            {
                var key = trimmedLine.Substring(0, separatorIndex).Trim();
                var value = trimmedLine.Substring(separatorIndex + 1).Trim();
                    
                // Remove inline comments
                var commentIndex = value.IndexOf(';');
                if (commentIndex >= 0)
                    value = value.Substring(0, commentIndex).Trim();
                    
                result[currentSection][key] = value;
            }
        }

        return result;
    }

    /// <summary>
    /// Generate an INI file from a nested dictionary
    /// </summary>
    public static void GenerateIniFile(string filePath, Dictionary<string, Dictionary<string, string>> settings)
    {
        var sb = new StringBuilder();
            
        foreach (var section in settings)
        {
            sb.AppendLine($"[{section.Key}]");
            foreach (var property in section.Value)
            {
                sb.AppendLine($"{property.Key}={property.Value}");
            }
            sb.AppendLine();
        }

        File.WriteAllText(filePath, sb.ToString());
    }

    /// <summary>
    /// Generate Engine.ini file from a settings preset
    /// </summary>
    public static void GenerateEngineIniFromPreset(SettingsPreset preset, string filePath = "../Engine.ini")
    {
        var iniData = new Dictionary<string, Dictionary<string, string>>();

        foreach (var setting in preset.Settings)
        {
            var parts = setting.Key.Split('.');
            if (parts.Length == 2)
            {
                var section = parts[0];
                var name = parts[1];
                    
                if (!iniData.ContainsKey(section))
                    iniData[section] = new Dictionary<string, string>();
                    
                iniData[section][name] = setting.Value;
            }
        }

        GenerateIniFile(filePath, iniData);
    }

    /// <summary>
    /// Generate recommended settings based on system info
    /// </summary>
    public static SettingsPreset GenerateRecommendedSettings(SystemInfo systemInfo, Database? db = null)
    {
        // Get the performance tier based on system info
        var tier = System.GetPerformanceTier(systemInfo);

        // Try to find a preset matching the tier in the database
        if (db != null)
        {
            var presets = db.GetAllPresets();
            foreach (var preset in presets)
            {
                if (preset.Name == tier)
                {
                    // Found a matching preset, return it with updated description
                    return new SettingsPreset
                    {
                        Id = preset.Id,
                        Name = "Recommended",
                        Description = $"Recommended settings for your system: {systemInfo.GpuName} with {systemInfo.GpuVramMb} VRAM",
                        CreatedAt = DateTime.Now.ToString("o"),
                        Settings = preset.Settings
                    };
                }
            }
        }

        // Fallback to hardcoded settings if no preset found
        var settings = new Dictionary<string, string>
        {
            ["SystemSettings.r.VSync"] = "0",
            ["RenderingThread.bAllowThreadedRendering"] = "True",
            ["RenderingThread.bAllowAsyncRenderThreadUpdates"] = "True",
            ["Engine.InputSettings.RawMouseInputEnabled"] = "1",
            ["Engine.InputSettings.bEnableMouseSmoothing"] = "False"
        };

        // Adjust settings based on VRAM
        var vramMb = systemInfo.GpuVramMb;
        var streamingPoolSize = vramMb > 8000 ? 12288 :
            vramMb > 6000 ? 8192 :
            vramMb > 4000 ? 4096 : 2048;

        settings["SystemSettings.r.Streaming.PoolSize"] = streamingPoolSize.ToString();

        // Adjust settings based on CPU cores
        var cpuCores = systemInfo.CpuCores;
        if (cpuCores >= 8)
        {
            settings["SystemSettings.r.AllowMultiThreadedShaderCreation"] = "1";
            settings["ShaderCompiler.bAllowCompilingThroughWorkerThreads"] = "True";
            settings["ShaderCompiler.NumUnusedShaderCompilingThreads"] = "3";
        }
        else if (cpuCores >= 4)
        {
            settings["SystemSettings.r.AllowMultiThreadedShaderCreation"] = "1";
            settings["ShaderCompiler.bAllowCompilingThroughWorkerThreads"] = "True";
            settings["ShaderCompiler.NumUnusedShaderCompilingThreads"] = "1";
        }
        else
        {
            settings["SystemSettings.r.AllowMultiThreadedShaderCreation"] = "0";
            settings["ShaderCompiler.bAllowCompilingThroughWorkerThreads"] = "False";
        }

        return new SettingsPreset
        {
            Name = "Recommended",
            Description = $"Recommended settings for your system: {systemInfo.GpuName} with {systemInfo.GpuVramMb} VRAM",
            Settings = settings
        };
    }
}