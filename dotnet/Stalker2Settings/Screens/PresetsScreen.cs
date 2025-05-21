using System.Collections.ObjectModel;
using Stalker2Settings.Models;
using Terminal.Gui;

namespace Stalker2Settings.Screens;

/// <summary>
/// UI component for managing presets
/// </summary>
public class PresetsScreen
{
    private readonly Database _db;
    private readonly List<Setting> _settings;
    private ObservableCollection<SettingsPreset> _presets;
    private SettingsPreset? _currentPreset;

    /// <summary>
    /// Initialize the presets UI
    /// </summary>
    public PresetsScreen(Database db, List<Setting> settings, SettingsPreset? currentPreset = null)
    {
        _db = db;
        _settings = settings;
        _currentPreset = currentPreset;
        var presets = _db.GetAllPresets();
        _presets = new ObservableCollection<SettingsPreset>(presets);
    }

    /// <summary>
    /// Show the presets list
    /// </summary>
    public void ShowPresetsList()
    {
        // Clear existing windows
        Application.Top.RemoveAll();
        Application.Top.Add(new StatusBar([
            new(Key.F1, "~F1~ Help", ShowHelp),
            new(Key.F2, "~F2~ Menu", () => Application.RequestStop()),
            new(Key.F10, "~F10~ Quit", () => Environment.Exit(0))
        ]));

        var win = new Window { Title = "Presets", X = 0, Y = 0, Width = Dim.Fill(), Height = Dim.Fill() - 1 };
        Application.Top.Add(win);

        // Create presets list view
        var listView = new ListView { X = 0, Y = 0, Width = Dim.Fill(), Height = Dim.Fill() - 4, AllowsMarking = false };
        var sources = _presets.Select(p => p.Name).ToList();
        listView.SetSource(new ObservableCollection<string>(sources));
        listView.OpenSelectedItem += (sender, args) =>
        {
            if (args.Item >= 0 && args.Item < _presets.Count)
                ShowPresetDetail(_presets[args.Item]);
        };
        win.Add(listView);

        // Create buttons
        var buttonY = Pos.Bottom(win) - 4;

        var newButton = new Button
        {
            Text = "New Preset",
            X = 1,
            Y = buttonY
        };
        newButton.Accepting += (s, e) => CreateNewPreset();
        win.Add(newButton);

        var applyButton = new Button
        {
            Text = "Apply Preset",
            X = Pos.Right(newButton) + 2,
            Y = buttonY
        };
        applyButton.Accepting += (s, e) =>
        {
            if (listView.SelectedItem >= 0 && listView.SelectedItem < _presets.Count)
            {
                ApplyPreset(_presets[listView.SelectedItem]);
            }
            else
            {
                MessageBox.ErrorQuery(
                    title: "Error",
                    message: "Please select a preset to apply.",
                    buttons: ["OK"]);
            }
        };
        win.Add(applyButton);

        var deleteButton = new Button
        {
            Text = "Delete Preset",
            X = Pos.Right(applyButton) + 2,
            Y = buttonY
        };
        deleteButton.Accepting += (s, e) =>
        {
            if (listView.SelectedItem >= 0 && listView.SelectedItem < _presets.Count)
            {
                DeletePreset(_presets[listView.SelectedItem]);
                _presets = new ObservableCollection<SettingsPreset>(_db.GetAllPresets());
                var names = _presets.Select(p => p.Name).ToList();
                listView.SetSource(new ObservableCollection<string>(names));
            }
            else
            {
                MessageBox.ErrorQuery(
                    title: "Error",
                    message: "Please select a preset to delete.",
                    buttons: ["OK"]);
            }
        };
        win.Add(deleteButton);

        var backButton = new Button
        {
            Text = "Back",
            X = Pos.Center(),
            Y = buttonY + 1
        };
        backButton.Accepting += (s, e) => Application.RequestStop();
        win.Add(backButton);

        // Run the dialog
        Application.Run(Application.Top);
    }

    /// <summary>
    /// Show preset detail for viewing/editing
    /// </summary>
    private void ShowPresetDetail(SettingsPreset preset)
    {
        var dialog = new Dialog
        {
            Title = $"Preset: {preset.Name}",
            Width = 70,
            Height = 20
        };

        // Add preset information
        var y = 1;
        dialog.Add(new Label { Text = $"Name: {preset.Name}", X = 1, Y = y++ });
        dialog.Add(new Label { Text = $"Description: {preset.Description}", X = 1, Y = y++ });
        dialog.Add(new Label { Text = $"Created: {preset.CreatedAt}", X = 1, Y = y++ });
        dialog.Add(new Label { Text = $"Settings: {preset.Settings.Count}", X = 1, Y = y++ });

        // Add settings list
        var settingsText = string.Join("\n", preset.Settings
            .OrderBy(s => s.Key)
            .Take(10) // Show only first 10 settings to avoid dialog being too large
            .Select(s => $"{s.Key} = {s.Value}"));

        if (preset.Settings.Count > 10)
            settingsText += $"\n... and {preset.Settings.Count - 10} more settings";

        var settingsView = new TextView
        {
            X = 1,
            Y = y,
            Width = Dim.Fill() - 2,
            Height = 10,
            ReadOnly = true,
            Text = settingsText
        };
        dialog.Add(settingsView);

        // Add buttons
        var applyBtn = new Button { Text = "Apply" };
        applyBtn.Accepting += (s, e) =>
        {
            ApplyPreset(preset);
            Application.RequestStop();
        };
        dialog.AddButton(applyBtn);

        var editBtn = new Button { Text = "Edit" };
        editBtn.Accepting += (s, e) =>
        {
            EditPreset(preset);
            Application.RequestStop();
        };
        dialog.AddButton(editBtn);

        var closeBtn = new Button { Text = "Close" };
        closeBtn.Accepting += (s, e) => { Application.RequestStop(); };
        dialog.AddButton(closeBtn);

        Application.Run(dialog);
    }

    /// <summary>
    /// Create a new preset
    /// </summary>
    private void CreateNewPreset()
    {
        var dialog = new Dialog { Title = "Create New Preset", Width = 60, Height = 15 };

        // Add name and description fields
        dialog.Add(new Label { X = 1, Y = 1, Title = "Name:" });
        var nameField = new TextField { X = 15, Y = 1, Width = 40 };
        dialog.Add(nameField);

        dialog.Add(new Label { X = 1, Y = 3, Title = "Description:" });
        var descriptionField = new TextField { X = 15, Y = 3, Width = 40 };
        dialog.Add(descriptionField);

        dialog.Add(new Label { X = 1, Y = 5, Title = "Base on:" });
        var baseOptions = new List<string> { "Current Settings" };
        baseOptions.AddRange(_presets.Select(p => p.Name));
        var baseRadio = new RadioGroup { X = 15, Y = 5, RadioLabels = baseOptions.ToArray() };
        dialog.Add(baseRadio);

        // Add buttons
        var btnCreate = new Button { Text = "Create" };
        btnCreate.Accepting += (s, e) =>
        {
            var name = nameField.Text;
            var description = descriptionField.Text;

            if (string.IsNullOrWhiteSpace(name))
            {
                MessageBox.ErrorQuery("Error", "Please enter a name for the preset.", "OK");
                return;
            }

            // Create new preset
            var preset = new SettingsPreset
            {
                Name = name,
                Description = description ?? "",
                Settings = new Dictionary<string, string>()
            };

            // Base on selected option
            if (baseRadio.SelectedItem == 0)
            {
                // Base on current settings
                foreach (var setting in _settings)
                {
                    var key = $"{setting.Section}.{setting.Name}";
                    preset.Settings[key] = setting.CurrentValue;
                }
            }
            else if (baseRadio.SelectedItem > 0 && baseRadio.SelectedItem <= _presets.Count)
            {
                // Base on selected preset
                var basePreset = _presets[baseRadio.SelectedItem - 1];
                preset.Settings = new Dictionary<string, string>(basePreset.Settings);
            }

            // Save preset
            _db.SavePreset(preset);
            _presets = new ObservableCollection<SettingsPreset>(_db.GetAllPresets());

            MessageBox.Query("Success", "Preset created successfully.", "OK");
            Application.RequestStop();
        };
        dialog.AddButton(btnCreate);
        
        var btnCancel = new Button { Text = "Cancel" };
        btnCancel.Accepting += (s, e) => { Application.RequestStop(); };
        dialog.AddButton(btnCancel);

        Application.Run(dialog);
    }

    /// <summary>
    /// Edit an existing preset
    /// </summary>
    private void EditPreset(SettingsPreset preset)
    {
        var dialog = new Dialog{ Width = 60, Height = 15, Title = $"Edit Preset: {preset.Name}" };

        // Add name and description fields
        dialog.Add(new Label { X = 1, Y = 1, Title = "Name:"});
        var nameField = new TextField
        {
            X = 15,
            Y = 1,
            Width = 40,
            Text = preset.Name
        };
        dialog.Add(nameField);

        dialog.Add(new Label { X = 1, Y = 3, Title = "Description:" });
        var descriptionField = new TextField
        {
            X = 15,
            Y = 3,
            Width = 40,
            Text = preset.Description
        };
        dialog.Add(descriptionField);

        var btnSave = new Button { Text = "Save" };
        btnSave.Accepting += (s, e) =>
        {
            var name = nameField.Text;
            var description = descriptionField.Text;

            if (string.IsNullOrWhiteSpace(name))
            {
                MessageBox.ErrorQuery("Error", "Please enter a name for the preset.", "OK");
                return;
            }

            // Update preset
            preset.Name = name;
            preset.Description = description ?? "";

            // Save preset
            _db.SavePreset(preset);
            _presets = new ObservableCollection<SettingsPreset>(_db.GetAllPresets());

            MessageBox.Query("Success", "Preset updated successfully.", "OK");
            Application.RequestStop();
        };
        dialog.AddButton(btnSave);

        var btnCancel = new Button { Text = "Cancel" };
        btnCancel.Accepting += (s, e) => { Application.RequestStop(); };
        dialog.AddButton(btnCancel);

        Application.Run(dialog);
    }

    /// <summary>
    /// Apply a preset to the current settings
    /// </summary>
    private void ApplyPreset(SettingsPreset preset)
    {
        var result = MessageBox.Query(
            "Apply Preset",
            $"Are you sure you want to apply the preset '{preset.Name}'?\n\nThis will overwrite your current settings.",
            "Apply", "Cancel");

        if (result == 0) // Apply
        {
            // Update settings with values from the preset
            foreach (var setting in _settings)
            {
                var key = $"{setting.Section}.{setting.Name}";
                if (preset.Settings.TryGetValue(key, out var value))
                {
                    setting.CurrentValue = value;
                    _db.SaveSetting(setting);
                }
            }

            // Set as current preset
            _currentPreset = preset;

            // Generate Engine.ini file
            try
            {
                Config.GenerateEngineIniFromPreset(preset);
                MessageBox.Query("Success", $"Applied preset '{preset.Name}' and generated Engine.ini", "OK");
            }
            catch (Exception ex)
            {
                MessageBox.ErrorQuery("Error", $"Error generating Engine.ini: {ex.Message}", "OK");
            }
        }
    }

    /// <summary>
    /// Delete a preset
    /// </summary>
    private void DeletePreset(SettingsPreset preset)
    {
        var result = MessageBox.Query(
            "Delete Preset",
            $"Are you sure you want to delete the preset '{preset.Name}'?\n\nThis action cannot be undone.",
            "Delete", "Cancel");

        if (result == 0) // Delete
        {
            if (preset.Id.HasValue)
            {
                _db.DeletePreset(preset.Id.Value);
                MessageBox.Query("Success", $"Preset '{preset.Name}' deleted successfully.", "OK");
            }
            else
            {
                MessageBox.ErrorQuery("Error", "Cannot delete preset without an ID.", "OK");
            }
        }
    }

    /// <summary>
    /// Show help information
    /// </summary>
    private void ShowHelp()
    {
        MessageBox.Query(
            "Presets Help",
            "Presets List\n\n" +
            "This screen shows all available presets for STALKER 2 settings.\n\n" +
            "- Select a preset and press Enter to view/edit it\n" +
            "- Use the buttons to create, apply, or delete presets\n" +
            "- Press F2 to return to the main menu\n" +
            "- Press F10 to quit the application",
            "OK");
    }
}