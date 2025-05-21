using Stalker2Settings;
using Stalker2Settings.Screens;

try
{
    var db = new Database(recreate: false);
    new MainScreen(db).Run();
    db.Close();
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}