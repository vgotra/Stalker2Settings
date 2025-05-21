using Microsoft.Data.Sqlite;
using Stalker2Settings.Models;

namespace Stalker2Settings;

/// <summary>
/// Handles database operations
/// </summary>
public class Database
{
    private readonly SqliteConnection _connection;

    /// <summary>
    /// Initialize the database
    /// </summary>
    public Database(string dbPath = "settings.db", bool recreate = false)
    {
        if (recreate && File.Exists(dbPath))
        {
            File.Delete(dbPath);
        }

        var connectionString = $"Data Source={dbPath}";
        _connection = new SqliteConnection(connectionString);
        _connection.Open();

        InitializeDatabase();
    }

    /// <summary>
    /// Initialize the database schema
    /// </summary>
    private void InitializeDatabase()
    {
        using var command = _connection.CreateCommand();
            
        // Create settings table if it doesn't exist
        command.CommandText = @"
                CREATE TABLE IF NOT EXISTS settings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    section TEXT NOT NULL,
                    description TEXT NOT NULL,
                    value_type TEXT NOT NULL,
                    current_value TEXT NOT NULL,
                    default_value TEXT NOT NULL,
                    min_value TEXT,
                    max_value TEXT,
                    impact TEXT NOT NULL,
                    enum_options TEXT
                );
            ";
        command.ExecuteNonQuery();

        // Create presets table if it doesn't exist
        command.CommandText = @"
                CREATE TABLE IF NOT EXISTS presets (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    description TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
            ";
        command.ExecuteNonQuery();

        // Create preset_settings table if it doesn't exist
        command.CommandText = @"
                CREATE TABLE IF NOT EXISTS preset_settings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    preset_id INTEGER NOT NULL,
                    setting_key TEXT NOT NULL,
                    setting_value TEXT NOT NULL,
                    FOREIGN KEY (preset_id) REFERENCES presets (id) ON DELETE CASCADE
                );
            ";
        command.ExecuteNonQuery();

        // Insert default settings if the settings table is empty
        command.CommandText = "SELECT COUNT(*) FROM settings;";
        var count = Convert.ToInt32(command.ExecuteScalar());

        if (count == 0)
        {
            InsertDefaultSettings();
        }
    }

    /// <summary>
    /// Insert default settings into the database
    /// </summary>
    private void InsertDefaultSettings()
    {
        var defaultSettings = new List<Setting>
        {
            new()
            {
                Name = "r.VSync",
                Section = "SystemSettings",
                Description = "Vertical synchronization",
                ValueType = SettingValueType.Boolean,
                CurrentValue = "0",
                DefaultValue = "0",
                Impact = "Performance"
            },
            new()
            {
                Name = "r.Streaming.PoolSize",
                Section = "SystemSettings",
                Description = "Texture streaming pool size in MB",
                ValueType = SettingValueType.Integer,
                CurrentValue = "12288",
                DefaultValue = "1024",
                MinValue = "1024",
                MaxValue = "16384",
                Impact = "Visual"
            },
            new()
            {
                Name = "r.PSOWarmup.WarmupMaterials",
                Section = "SystemSettings",
                Description = "Warm up materials for shader compilation",
                ValueType = SettingValueType.Boolean,
                CurrentValue = "1",
                DefaultValue = "1",
                Impact = "Performance"
            },
            new()
            {
                Name = "bAllowThreadedRendering",
                Section = "RenderingThread",
                Description = "Allow threaded rendering",
                ValueType = SettingValueType.Boolean,
                CurrentValue = "True",
                DefaultValue = "True",
                Impact = "Performance"
            },
            new()
            {
                Name = "bAllowAsyncRenderThreadUpdates",
                Section = "RenderingThread",
                Description = "Allow async render thread updates",
                ValueType = SettingValueType.Boolean,
                CurrentValue = "True",
                DefaultValue = "True",
                Impact = "Performance"
            },
            new()
            {
                Name = "RawMouseInputEnabled",
                Section = "Engine.InputSettings",
                Description = "Enable raw mouse input",
                ValueType = SettingValueType.Boolean,
                CurrentValue = "1",
                DefaultValue = "1",
                Impact = "Control"
            },
            new()
            {
                Name = "bEnableMouseSmoothing",
                Section = "Engine.InputSettings",
                Description = "Enable mouse smoothing",
                ValueType = SettingValueType.Boolean,
                CurrentValue = "False",
                DefaultValue = "False",
                Impact = "Control"
            }
        };

        using var transaction = _connection.BeginTransaction();
            
        foreach (var setting in defaultSettings)
        {
            using var command = _connection.CreateCommand();
            command.CommandText = @"
                    INSERT INTO settings (name, section, description, value_type, current_value, default_value, min_value, max_value, impact, enum_options)
                    VALUES (@name, @section, @description, @value_type, @current_value, @default_value, @min_value, @max_value, @impact, @enum_options);
                ";
            command.Parameters.AddWithValue("@name", setting.Name);
            command.Parameters.AddWithValue("@section", setting.Section);
            command.Parameters.AddWithValue("@description", setting.Description);
            command.Parameters.AddWithValue("@value_type", setting.ValueType.ToString());
            command.Parameters.AddWithValue("@current_value", setting.CurrentValue);
            command.Parameters.AddWithValue("@default_value", setting.DefaultValue);
            command.Parameters.AddWithValue("@min_value", setting.MinValue ?? (object)DBNull.Value);
            command.Parameters.AddWithValue("@max_value", setting.MaxValue ?? (object)DBNull.Value);
            command.Parameters.AddWithValue("@impact", setting.Impact);
            command.Parameters.AddWithValue("@enum_options", setting.EnumOptions != null ? string.Join(",", setting.EnumOptions) : DBNull.Value);
            command.ExecuteNonQuery();
        }

        // Insert default presets
        InsertDefaultPresets(transaction);

        transaction.Commit();
    }

    /// <summary>
    /// Insert default presets into the database
    /// </summary>
    private void InsertDefaultPresets(SqliteTransaction transaction)
    {
        var presets = new List<SettingsPreset>
        {
            new()
            {
                Name = "Ultra",
                Description = "Maximum quality settings for high-end systems",
                Settings = new Dictionary<string, string>
                {
                    ["SystemSettings.r.VSync"] = "0",
                    ["SystemSettings.r.Streaming.PoolSize"] = "12288",
                    ["SystemSettings.r.PSOWarmup.WarmupMaterials"] = "1",
                    ["RenderingThread.bAllowThreadedRendering"] = "True",
                    ["RenderingThread.bAllowAsyncRenderThreadUpdates"] = "True",
                    ["Engine.InputSettings.RawMouseInputEnabled"] = "1",
                    ["Engine.InputSettings.bEnableMouseSmoothing"] = "False",
                    ["SystemSettings.r.AllowMultiThreadedShaderCreation"] = "1",
                    ["ShaderCompiler.bAllowCompilingThroughWorkerThreads"] = "True",
                    ["ShaderCompiler.NumUnusedShaderCompilingThreads"] = "3"
                }
            },
            new()
            {
                Name = "High",
                Description = "High quality settings for mid to high-end systems",
                Settings = new Dictionary<string, string>
                {
                    ["SystemSettings.r.VSync"] = "0",
                    ["SystemSettings.r.Streaming.PoolSize"] = "8192",
                    ["SystemSettings.r.PSOWarmup.WarmupMaterials"] = "1",
                    ["RenderingThread.bAllowThreadedRendering"] = "True",
                    ["RenderingThread.bAllowAsyncRenderThreadUpdates"] = "True",
                    ["Engine.InputSettings.RawMouseInputEnabled"] = "1",
                    ["Engine.InputSettings.bEnableMouseSmoothing"] = "False",
                    ["SystemSettings.r.AllowMultiThreadedShaderCreation"] = "1",
                    ["ShaderCompiler.bAllowCompilingThroughWorkerThreads"] = "True",
                    ["ShaderCompiler.NumUnusedShaderCompilingThreads"] = "2"
                }
            },
            new()
            {
                Name = "Medium",
                Description = "Balanced settings for mid-range systems",
                Settings = new Dictionary<string, string>
                {
                    ["SystemSettings.r.VSync"] = "0",
                    ["SystemSettings.r.Streaming.PoolSize"] = "4096",
                    ["SystemSettings.r.PSOWarmup.WarmupMaterials"] = "1",
                    ["RenderingThread.bAllowThreadedRendering"] = "True",
                    ["RenderingThread.bAllowAsyncRenderThreadUpdates"] = "True",
                    ["Engine.InputSettings.RawMouseInputEnabled"] = "1",
                    ["Engine.InputSettings.bEnableMouseSmoothing"] = "False",
                    ["SystemSettings.r.AllowMultiThreadedShaderCreation"] = "1",
                    ["ShaderCompiler.bAllowCompilingThroughWorkerThreads"] = "True",
                    ["ShaderCompiler.NumUnusedShaderCompilingThreads"] = "1"
                }
            },
            new()
            {
                Name = "Low",
                Description = "Performance-focused settings for lower-end systems",
                Settings = new Dictionary<string, string>
                {
                    ["SystemSettings.r.VSync"] = "0",
                    ["SystemSettings.r.Streaming.PoolSize"] = "2048",
                    ["SystemSettings.r.PSOWarmup.WarmupMaterials"] = "1",
                    ["RenderingThread.bAllowThreadedRendering"] = "True",
                    ["RenderingThread.bAllowAsyncRenderThreadUpdates"] = "True",
                    ["Engine.InputSettings.RawMouseInputEnabled"] = "1",
                    ["Engine.InputSettings.bEnableMouseSmoothing"] = "False",
                    ["SystemSettings.r.AllowMultiThreadedShaderCreation"] = "0",
                    ["ShaderCompiler.bAllowCompilingThroughWorkerThreads"] = "False"
                }
            },
            new()
            {
                Name = "Minimum",
                Description = "Minimum settings for very low-end systems",
                Settings = new Dictionary<string, string>
                {
                    ["SystemSettings.r.VSync"] = "0",
                    ["SystemSettings.r.Streaming.PoolSize"] = "1024",
                    ["SystemSettings.r.PSOWarmup.WarmupMaterials"] = "1",
                    ["RenderingThread.bAllowThreadedRendering"] = "False",
                    ["RenderingThread.bAllowAsyncRenderThreadUpdates"] = "False",
                    ["Engine.InputSettings.RawMouseInputEnabled"] = "1",
                    ["Engine.InputSettings.bEnableMouseSmoothing"] = "False",
                    ["SystemSettings.r.AllowMultiThreadedShaderCreation"] = "0",
                    ["ShaderCompiler.bAllowCompilingThroughWorkerThreads"] = "False"
                }
            }
        };

        foreach (var preset in presets)
        {
            // Insert preset
            using (var command = _connection.CreateCommand())
            {
                command.Transaction = transaction;
                command.CommandText = @"
                        INSERT INTO presets (name, description, created_at)
                        VALUES (@name, @description, @created_at);
                        SELECT last_insert_rowid();
                    ";
                command.Parameters.AddWithValue("@name", preset.Name);
                command.Parameters.AddWithValue("@description", preset.Description);
                command.Parameters.AddWithValue("@created_at", preset.CreatedAt);
                var presetId = Convert.ToInt64(command.ExecuteScalar());

                // Insert preset settings
                foreach (var setting in preset.Settings)
                {
                    using var settingCommand = _connection.CreateCommand();
                    settingCommand.Transaction = transaction;
                    settingCommand.CommandText = @"
                            INSERT INTO preset_settings (preset_id, setting_key, setting_value)
                            VALUES (@preset_id, @setting_key, @setting_value);
                        ";
                    settingCommand.Parameters.AddWithValue("@preset_id", presetId);
                    settingCommand.Parameters.AddWithValue("@setting_key", setting.Key);
                    settingCommand.Parameters.AddWithValue("@setting_value", setting.Value);
                    settingCommand.ExecuteNonQuery();
                }
            }
        }
    }

    /// <summary>
    /// Get all settings from the database
    /// </summary>
    public List<Setting> GetAllSettings()
    {
        var settings = new List<Setting>();

        using var command = _connection.CreateCommand();
        command.CommandText = "SELECT * FROM settings ORDER BY section, name;";

        using var reader = command.ExecuteReader();
        while (reader.Read())
        {
            var setting = new Setting
            {
                Name = reader.GetString(reader.GetOrdinal("name")),
                Section = reader.GetString(reader.GetOrdinal("section")),
                Description = reader.GetString(reader.GetOrdinal("description")),
                ValueType = Enum.Parse<SettingValueType>(reader.GetString(reader.GetOrdinal("value_type"))),
                CurrentValue = reader.GetString(reader.GetOrdinal("current_value")),
                DefaultValue = reader.GetString(reader.GetOrdinal("default_value")),
                Impact = reader.GetString(reader.GetOrdinal("impact"))
            };

            var minValueOrdinal = reader.GetOrdinal("min_value");
            if (!reader.IsDBNull(minValueOrdinal))
            {
                setting.MinValue = reader.GetString(minValueOrdinal);
            }

            var maxValueOrdinal = reader.GetOrdinal("max_value");
            if (!reader.IsDBNull(maxValueOrdinal))
            {
                setting.MaxValue = reader.GetString(maxValueOrdinal);
            }

            var enumOptionsOrdinal = reader.GetOrdinal("enum_options");
            if (!reader.IsDBNull(enumOptionsOrdinal))
            {
                var options = reader.GetString(enumOptionsOrdinal);
                setting.EnumOptions = new List<string>(options.Split(','));
            }

            settings.Add(setting);
        }

        return settings;
    }

    /// <summary>
    /// Save a setting to the database
    /// </summary>
    public void SaveSetting(Setting setting)
    {
        using var command = _connection.CreateCommand();
        command.CommandText = @"
                UPDATE settings
                SET current_value = @current_value
                WHERE name = @name AND section = @section;
            ";
        command.Parameters.AddWithValue("@current_value", setting.CurrentValue);
        command.Parameters.AddWithValue("@name", setting.Name);
        command.Parameters.AddWithValue("@section", setting.Section);
        command.ExecuteNonQuery();
    }

    /// <summary>
    /// Get all presets from the database
    /// </summary>
    public List<SettingsPreset> GetAllPresets()
    {
        var presets = new List<SettingsPreset>();

        using var command = _connection.CreateCommand();
        command.CommandText = "SELECT * FROM presets ORDER BY name;";

        using var reader = command.ExecuteReader();
        while (reader.Read())
        {
            var preset = new SettingsPreset
            {
                Id = reader.GetInt64(reader.GetOrdinal("id")),
                Name = reader.GetString(reader.GetOrdinal("name")),
                Description = reader.GetString(reader.GetOrdinal("description")),
                CreatedAt = reader.GetString(reader.GetOrdinal("created_at")),
                Settings = new Dictionary<string, string>()
            };

            // Get settings for this preset
            using var settingsCommand = _connection.CreateCommand();
            settingsCommand.CommandText = "SELECT setting_key, setting_value FROM preset_settings WHERE preset_id = @preset_id;";
            settingsCommand.Parameters.AddWithValue("@preset_id", preset.Id);

            using var settingsReader = settingsCommand.ExecuteReader();
            while (settingsReader.Read())
            {
                var key = settingsReader.GetString(0);
                var value = settingsReader.GetString(1);
                preset.Settings[key] = value;
            }

            presets.Add(preset);
        }

        return presets;
    }

    /// <summary>
    /// Save a preset to the database
    /// </summary>
    public void SavePreset(SettingsPreset preset)
    {
        using var transaction = _connection.BeginTransaction();

        try
        {
            long presetId;

            if (preset.Id.HasValue)
            {
                // Update existing preset
                using var updateCommand = _connection.CreateCommand();
                updateCommand.Transaction = transaction;
                updateCommand.CommandText = @"
                        UPDATE presets
                        SET name = @name, description = @description
                        WHERE id = @id;
                    ";
                updateCommand.Parameters.AddWithValue("@name", preset.Name);
                updateCommand.Parameters.AddWithValue("@description", preset.Description);
                updateCommand.Parameters.AddWithValue("@id", preset.Id.Value);
                updateCommand.ExecuteNonQuery();

                presetId = preset.Id.Value;

                // Delete existing settings
                using var deleteCommand = _connection.CreateCommand();
                deleteCommand.Transaction = transaction;
                deleteCommand.CommandText = "DELETE FROM preset_settings WHERE preset_id = @preset_id;";
                deleteCommand.Parameters.AddWithValue("@preset_id", presetId);
                deleteCommand.ExecuteNonQuery();
            }
            else
            {
                // Insert new preset
                using var insertCommand = _connection.CreateCommand();
                insertCommand.Transaction = transaction;
                insertCommand.CommandText = @"
                        INSERT INTO presets (name, description, created_at)
                        VALUES (@name, @description, @created_at);
                        SELECT last_insert_rowid();
                    ";
                insertCommand.Parameters.AddWithValue("@name", preset.Name);
                insertCommand.Parameters.AddWithValue("@description", preset.Description);
                insertCommand.Parameters.AddWithValue("@created_at", preset.CreatedAt);
                presetId = Convert.ToInt64(insertCommand.ExecuteScalar());
            }

            // Insert settings
            foreach (var setting in preset.Settings)
            {
                using var settingCommand = _connection.CreateCommand();
                settingCommand.Transaction = transaction;
                settingCommand.CommandText = @"
                        INSERT INTO preset_settings (preset_id, setting_key, setting_value)
                        VALUES (@preset_id, @setting_key, @setting_value);
                    ";
                settingCommand.Parameters.AddWithValue("@preset_id", presetId);
                settingCommand.Parameters.AddWithValue("@setting_key", setting.Key);
                settingCommand.Parameters.AddWithValue("@setting_value", setting.Value);
                settingCommand.ExecuteNonQuery();
            }

            transaction.Commit();
        }
        catch
        {
            transaction.Rollback();
            throw;
        }
    }

    /// <summary>
    /// Delete a preset from the database
    /// </summary>
    public void DeletePreset(long presetId)
    {
        using var command = _connection.CreateCommand();
        command.CommandText = "DELETE FROM presets WHERE id = @id;";
        command.Parameters.AddWithValue("@id", presetId);
        command.ExecuteNonQuery();
    }

    /// <summary>
    /// Close the database connection
    /// </summary>
    public void Close()
    {
        _connection.Close();
    }
}