using System.CommandLine;
using Stalker2Settings.Screens;

namespace Stalker2Settings;

/// <summary>
/// Main program entry point
/// </summary>
class Program
{
    /// <summary>
    /// Main method
    /// </summary>
    static async Task<int> Main(string[] args)
    {
        // Create command line options
        var rootCommand = new RootCommand("Stalker 2 Settings Manager");
        var recreateDbOption = new Option<bool>("--recreatedb", "Recreate the database from scratch");
        recreateDbOption.AddAlias("-r");
        rootCommand.AddOption(recreateDbOption);

        rootCommand.SetHandler(recreateDb =>
        {
            try
            {
                var db = new Database(recreate: recreateDb);
                new MainScreen(db).Run();
                db.Close();
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error: {ex.Message}");
            }
        }, recreateDbOption);

        return await rootCommand.InvokeAsync(args);
    }
}
