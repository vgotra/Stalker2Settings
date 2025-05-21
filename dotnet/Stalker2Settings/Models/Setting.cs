namespace Stalker2Settings.Models;

/// <summary>
/// Represents the type of a setting value
/// </summary>
public enum SettingValueType
{
    Boolean,
    Integer,
    Float,
    String,
    Enum
}

/// <summary>
/// Represents a single setting with its metadata
/// </summary>
public class Setting
{
    public string Name { get; set; } = string.Empty;
    public string Section { get; set; } = string.Empty;
    public string Description { get; set; } = string.Empty;
    public SettingValueType ValueType { get; set; }
    public string CurrentValue { get; set; } = string.Empty;
    public string DefaultValue { get; set; } = string.Empty;
    public string? MinValue { get; set; }
    public string? MaxValue { get; set; }
    public string Impact { get; set; } = string.Empty; // Performance, Visual, etc.
    public List<string>? EnumOptions { get; set; }

    /// <summary>
    /// Check if the provided value is valid for this setting
    /// </summary>
    public bool IsValidValue(string value)
    {
        return ValueType switch
        {
            SettingValueType.Boolean => value == "0" || value == "1" || value == "True" || value == "False",
            SettingValueType.Integer => int.TryParse(value, out _),
            SettingValueType.Float => float.TryParse(value, out _),
            SettingValueType.String => true,
            SettingValueType.Enum => EnumOptions != null && EnumOptions.Contains(value),
            _ => false
        };
    }
}