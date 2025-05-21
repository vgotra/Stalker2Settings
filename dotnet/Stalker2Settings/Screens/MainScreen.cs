using Stalker2Settings.Models;
using Terminal.Gui;
using Attribute = Terminal.Gui.Attribute;

namespace Stalker2Settings.Screens;

/// <summary>
/// Main UI class for the application
/// </summary>
public class MainScreen
{
    private readonly Database _db;
    private readonly List<Setting> _settings;
    private readonly SystemInfo _systemInfo;
    private SettingsPreset? _currentPreset;

    /// <summary>
    /// Initialize the UI
    /// </summary>
    public MainScreen(Database db)
    {
        _db = db;
        _settings = _db.GetAllSettings();
        _systemInfo = System.GetSystemInfo();
        _currentPreset = null;
    }

    /// <summary>
    /// Run the application UI
    /// </summary>
    public void Run()
    {
        Application.Init();

        var top = new Toplevel
        {
            Title = "STALKER 2 Settings Manager",
            X = 0,
            Y = 0,
            Width = Dim.Fill(),
            Height = Dim.Fill() - 1, // Leave room for status bar
        };
        
        top.ColorScheme = new ColorScheme
        {
            Normal = new Attribute(Color.White, Color.Blue),
            Focus = new Attribute(Color.Black, Color.Gray),
            HotNormal = new Attribute(Color.BrightYellow, Color.Blue),
            HotFocus = new Attribute(Color.BrightYellow, Color.Gray)
        };;
        
        var statusBar = new StatusBar([
            new(Key.F1, "~F1~ Help", ShowHelp),
            new(Key.F2, "~F2~ Menu", ShowMainMenu),
            new(Key.F10, "~F10~ Quit", () => Application.RequestStop())
        ]);
        top.Add(statusBar);

        //ShowMainMenu();

        Application.Run(top);
    }

    /// <summary>
    /// Show the main menu
    /// </summary>
    private void ShowMainMenu()
    {
        // Create main window
        var win = new Window
        {
            Title = "STALKER 2 Settings Manager",
            X = 0,
            Y = 0,
            Width = Dim.Fill(),
            Height = Dim.Fill() - 1 // Leave room for status bar
        };
        win.Add(new StatusBar([
            new(Key.F1, "~F1~ Help", ShowHelp),
            new(Key.F10, "~F10~ Quit", () => Application.RequestStop())
        ]));
        Application.Run(win);

        // Create system info display
        var systemInfoLabel = new Label
        {
            Text = $"System: {_systemInfo.CpuName} with {_systemInfo.CpuCores} cores, {_systemInfo.RamGb}GB RAM, {_systemInfo.GpuName}",
            X = 0,
            Y = 0,
            Width = Dim.Fill()
        };
        win.Add(systemInfoLabel);

        var performanceTier = System.GetPerformanceTier(_systemInfo);
        var performanceLabel = new Label
        {
            Text = $"Performance tier: {performanceTier} (estimated VRAM: {_systemInfo.GpuVramMb}MB)",
            X = 0,
            Y = 1,
            Width = Dim.Fill()
        };
        win.Add(performanceLabel);

        // Create menu buttons
        var buttonY = 3;
        var editSettingsBtn = new Button
        {
            Text = "Edit Settings",
            X = Pos.Center(),
            Y = buttonY++
        };
        editSettingsBtn.Accepting += (s,e) => ShowSettingsList();
        win.Add(editSettingsBtn);

        var managePresetsBtn = new Button
        {
            Text = "Manage Presets",
            X = Pos.Center(),
            Y = buttonY++
        };
        managePresetsBtn.Accepting += (s,e) => ShowPresetsList();
        win.Add(managePresetsBtn);

        var generateRecommendedBtn = new Button
        {
            Text = "Generate Recommended Settings",
            X = Pos.Center(),
            Y = buttonY++
        };
        generateRecommendedBtn.Accepting += (s,e) => GenerateRecommendedSettings();
        win.Add(generateRecommendedBtn);

        var saveCurrentBtn = new Button
        {
            Text = "Save Current Settings",
            X = Pos.Center(),
            Y = buttonY++
        };
        saveCurrentBtn.Accepting += (s,e) => SaveCurrentSettings();
        win.Add(saveCurrentBtn);

        var quitBtn = new Button
        {
            Text = "Quit",
            X = Pos.Center(),
            Y = buttonY++
        };
        quitBtn.Accepting += (s,e) => Application.RequestStop();
        win.Add(quitBtn);
    }

    /// <summary>
    /// Show the settings list
    /// </summary>
    private void ShowSettingsList()
    {
        var settingsScreen = new SettingsScreen(_db, _settings);
        settingsScreen.ShowSettingsList();
    }

    /// <summary>
    /// Show the presets list
    /// </summary>
    private void ShowPresetsList()
    {
        var presetsScreen = new PresetsScreen(_db, _settings, _currentPreset);
        presetsScreen.ShowPresetsList();
    }

    /// <summary>
    /// Generate recommended settings based on system info
    /// </summary>
    private void GenerateRecommendedSettings()
    {
        var preset = Config.GenerateRecommendedSettings(_systemInfo, _db);

        var result = MessageBox.Query(
            title: "Recommended Settings",
            message:
            $@"Generated recommended settings for your system:
{_systemInfo.CpuName} with {_systemInfo.CpuCores} cores, {_systemInfo.RamGb}GB RAM

Performance tier: {System.GetPerformanceTier(_systemInfo)}

Do you want to apply these settings?",
            buttons: ["Apply", "Cancel"]);

        if (result == 0) // Apply
        {
            ApplyPreset(preset);
        }
    }

    /// <summary>
    /// Apply a preset to the current settings
    /// </summary>
    private void ApplyPreset(SettingsPreset preset)
    {
        // Update settings with values from the preset
        foreach (var setting in _settings)
        {
            var key = $"{setting.Section}.{setting.Name}";
            if (preset.Settings.TryGetValue(key, out var value))
            {
                setting.CurrentValue = value;
            }
        }

        // Set as current preset
        _currentPreset = preset;

        // Generate Engine.ini file
        try
        {
            Config.GenerateEngineIniFromPreset(preset);
            MessageBox.Query(
                title: "Success",
                message: $"Applied preset '{preset.Name}' and generated Engine.ini",
                buttons: ["OK"]);
        }
        catch (Exception ex)
        {
            MessageBox.ErrorQuery(
                title: "Error",
                message: $"Error generating Engine.ini: {ex.Message}",
                buttons: ["OK"]);
        }
    }

    /// <summary>
    /// Save current settings to Engine.ini
    /// </summary>
    private void SaveCurrentSettings()
    {
        // Create a preset from current settings
        var settings = new Dictionary<string, string>();

        foreach (var setting in _settings)
        {
            var key = $"{setting.Section}.{setting.Name}";
            settings[key] = setting.CurrentValue;
        }

        var preset = new SettingsPreset
        {
            Name = "Current",
            Description = "Current settings",
            Settings = settings
        };

        // Generate Engine.ini file
        try
        {
            Config.GenerateEngineIniFromPreset(preset);
            MessageBox.Query(
                title: "Success",
                message: "Settings saved to Engine.ini",
                buttons: ["OK"]);
        }
        catch (Exception ex)
        {
            MessageBox.ErrorQuery(
                title: "Error",
                message: $"Error generating Engine.ini: {ex.Message}",
                buttons: ["OK"]);
        }
    }

    /// <summary>
    /// Show help information
    /// </summary>
    private void ShowHelp()
    {
        MessageBox.Query(
            title: "Help",
            message: "STALKER 2 Settings Manager\n\n" +
                     "This application helps you manage settings for STALKER 2.\n\n" +
                     "- Edit Settings: View and modify individual settings\n" +
                     "- Manage Presets: Create, edit, and apply setting presets\n" +
                     "- Generate Recommended Settings: Create settings based on your hardware\n" +
                     "- Save Current Settings: Save current settings to Engine.ini\n\n" +
                     "Press F10 to quit the application.",
            buttons: ["OK"]);
    }
}