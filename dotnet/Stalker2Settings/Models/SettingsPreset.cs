namespace Stalker2Settings.Models;

/// <summary>
/// Represents a named collection of settings (a preset)
/// </summary>
public class SettingsPreset
{
    public long? Id { get; set; }
    public string Name { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
    public string CreatedAt { get; set; } = DateTime.Now.ToString("o");
    public Dictionary<string, string> Settings { get; set; } = new();

    public SettingsPreset()
    {
    }

    public SettingsPreset(string name, string description)
    {
        Name = name;
        Description = description;
        CreatedAt = DateTime.Now.ToString("o");
    }

    public SettingsPreset Clone()
    {
        return new SettingsPreset
        {
            Id = Id,
            Name = Name,
            Description = Description,
            CreatedAt = CreatedAt,
            Settings = new Dictionary<string, string>(Settings)
        };
    }
}