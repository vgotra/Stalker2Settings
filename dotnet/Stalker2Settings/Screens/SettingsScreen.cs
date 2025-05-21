using System.Collections.ObjectModel;
using Stalker2Settings.Models;
using Terminal.Gui;

namespace Stalker2Settings.Screens;

/// <summary>
/// UI component for managing settings
/// </summary>
public class SettingsScreen
{
    private readonly Database _db;
    private readonly ObservableCollection<Setting> _settings;

    /// <summary>
    /// Initialize the settings UI
    /// </summary>
    public SettingsScreen(Database db, List<Setting> settings)
    {
        _db = db;
        _settings = new ObservableCollection<Setting>(settings);
    }

    /// <summary>
    /// Show the settings list
    /// </summary>
    public void ShowSettingsList()
    {
        Application.Top.RemoveAll();
        Application.Top.Add(new StatusBar([
            new (Key.F1, "~F1~ Help", () => ShowHelp()),
            new (Key.F2, "~F2~ Menu", () => Application.RequestStop()),
            new (Key.F10, "~F10~ Quit", () => Environment.Exit(0))
        ]));

        // Create main window
        var win = new Window
        {
            Title = "Settings",
            X = 0,
            Y = 0,
            Width = Dim.Fill(),
            Height = Dim.Fill() - 1 // Leave room for status bar
        };
        Application.Top.Add(win);

        // Create settings list view
        var listView = new ListView
        {
            X = 0,
            Y = 0,
            Width = Dim.Fill(),
            Height = Dim.Fill() - 2,
            AllowsMarking = false
        };
        listView.SetSource(_settings);
        listView.OpenSelectedItem += (sender, args) => {
            if (args.Item >= 0 && args.Item < _settings.Count)
            {
                ShowSettingDetail(_settings[args.Item]);
            }
        };
        win.Add(listView);

        // Create back button
        var backButton = new Button
        {
            Text = "Back",
            X = Pos.Center(),
            Y = Pos.Bottom(win) - 3
        };
        backButton.Accepting += (s,e) => Application.RequestStop();
        win.Add(backButton);

        // Run the dialog
        Application.Run(Application.Top);
    }

    /// <summary>
    /// Show setting detail for editing
    /// </summary>
    private void ShowSettingDetail(Setting setting)
    {
        var dialog = new Dialog
        {
            Title = $"Edit Setting: {setting.Name}",
            Width = 60,
            Height = 15
        };

        // Add setting information
        var y = 1;
        dialog.Add(new Label { Text = $"Name: {setting.Name}", X = 1, Y = y++ });
        dialog.Add(new Label { Text = $"Section: {setting.Section}", X = 1, Y = y++ });
        dialog.Add(new Label { Text = $"Description: {setting.Description}", X = 1, Y = y++ });
        dialog.Add(new Label { Text = $"Type: {setting.ValueType}", X = 1, Y = y++ });
        dialog.Add(new Label { Text = $"Default Value: {setting.DefaultValue}", X = 1, Y = y++ });

        if (!string.IsNullOrEmpty(setting.MinValue))
            dialog.Add(new Label { Text = $"Min Value: {setting.MinValue}", X = 1, Y = y++ });

        if (!string.IsNullOrEmpty(setting.MaxValue))
            dialog.Add(new Label { Text = $"Max Value: {setting.MaxValue}", X = 1, Y = y++ });

        dialog.Add(new Label { Text = $"Impact: {setting.Impact}", X = 1, Y = y++ });
        dialog.Add(new Label { Text = "Current Value:", X = 1, Y = y });

        // Add value editor based on type
        View valueEditor;
        switch (setting.ValueType)
        {
            case SettingValueType.Boolean:
                var boolValue = setting.CurrentValue == "1" || setting.CurrentValue == "True";
                var radioGroup = new RadioGroup
                {
                    X = 1,
                    Y = y + 1,
                    RadioLabels = ["True", "False"]
                };
                radioGroup.SelectedItem = boolValue ? 0 : 1;
                valueEditor = radioGroup;
                break;

            case SettingValueType.Enum when setting.EnumOptions != null:
                var enumRadio = new RadioGroup
                {
                    X = 1,
                    Y = y + 1,
                    RadioLabels = setting.EnumOptions.ToArray()
                };
                var selectedIndex = setting.EnumOptions.IndexOf(setting.CurrentValue);
                enumRadio.SelectedItem = selectedIndex >= 0 ? selectedIndex : 0;
                valueEditor = enumRadio;
                break;

            case SettingValueType.Integer:
            case SettingValueType.Float:
            case SettingValueType.String:
            default:
                var textField = new TextField
                {
                    Text = setting.CurrentValue,
                    X = 15,
                    Y = y,
                    Width = 30
                };
                valueEditor = textField;
                break;
        }
        dialog.Add(valueEditor);

        // Add buttons
        var saveBtn = new Button { Text = "Save" };
        saveBtn.Accepting += (s,e) => {
            // Get value from editor
            string newValue;
            if (valueEditor is RadioGroup radioGroup)
            {
                if (setting.ValueType == SettingValueType.Boolean)
                {
                    newValue = radioGroup.SelectedItem == 0 ? "True" : "False";
                }
                else if (setting.ValueType == SettingValueType.Enum && setting.EnumOptions != null)
                {
                    newValue = setting.EnumOptions[radioGroup.SelectedItem];
                }
                else
                {
                    newValue = radioGroup.SelectedItem.ToString() ?? setting.CurrentValue;
                }
            }
            else if (valueEditor is TextField textField)
            {
                newValue = textField.Text ?? setting.CurrentValue;
            }
            else
            {
                newValue = setting.CurrentValue;
            }

            // Validate value
            if (!setting.IsValidValue(newValue))
            {
                MessageBox.ErrorQuery(
                    title: "Invalid Value", 
                    message: $"The value '{newValue}' is not valid for this setting.", 
                    buttons: ["OK"]);
                return;
            }

            // Update setting
            setting.CurrentValue = newValue;
            _db.SaveSetting(setting);

            MessageBox.Query(
                title: "Success", 
                message: "Setting updated successfully.", 
                buttons: ["OK"]);
            Application.RequestStop();
        };
        dialog.AddButton(saveBtn);

        var cancelBtn = new Button { Text = "Cancel" };
        cancelBtn.Accepting += (s,e) => {
            Application.RequestStop();
        };
        dialog.AddButton(cancelBtn);

        Application.Run(dialog);
    }

    /// <summary>
    /// Show help information
    /// </summary>
    private void ShowHelp()
    {
        MessageBox.Query(
            title: "Settings Help", 
            message: "Settings List\n\n" +
            "This screen shows all available settings for STALKER 2.\n\n" +
            "- Select a setting and press Enter to edit it\n" +
            "- Press F2 to return to the main menu\n" +
            "- Press F10 to quit the application",
            buttons: ["OK"]);
    }
}
