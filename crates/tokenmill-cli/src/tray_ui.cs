using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Web.Script.Serialization;
using System.Windows.Forms;

sealed class TokenmillTray : ApplicationContext
{
    readonly string executable, home;
    readonly FileStream instance;
    readonly NotifyIcon icon = new NotifyIcon();
    readonly ContextMenuStrip menu = new ContextMenuStrip();
    readonly Form window = new Form();
    readonly Label summary = new Label();
    readonly Timer timer = new Timer();
    readonly ToolStripMenuItem status, verification, routing, saver, mode, strict, compatible, error;
    string lastError;
    bool quitting;

    [STAThread]
    static int Main(string[] args)
    {
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        try {
            if (args.Length != 1) throw new Exception("Start Tokenmill with tokenmill-tray.exe.");
            using (var app = new TokenmillTray(args[0])) Application.Run(app);
            return 0;
        } catch (Exception exception) {
            MessageBox.Show(exception.Message, "Tokenmill startup failed", MessageBoxButtons.OK, MessageBoxIcon.Error);
            return 1;
        }
    }

    TokenmillTray(string program)
    {
        executable = program;
        home = Environment.GetEnvironmentVariable("TOKENMILL_HOME");
        instance = new FileStream(Path.Combine(home, "tray-ui.lock"), FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None);
        window.Text = "Tokenmill controls";
        window.ClientSize = new Size(460, 215);
        window.StartPosition = FormStartPosition.CenterScreen;
        window.FormBorderStyle = FormBorderStyle.FixedDialog;
        window.MaximizeBox = false;
        window.AutoScaleMode = AutoScaleMode.Dpi;
        window.Font = new Font("Segoe UI", 10);
        window.Icon = SystemIcons.Application;
        summary.SetBounds(20, 20, 420, 45);
        window.Controls.Add(summary);
        var scope = new Label { Text = "Controls acp-context-prompt only.\r\nExternal GitHub Copilot requests are unaffected." };
        scope.SetBounds(20, 75, 420, 50);
        window.Controls.Add(scope);
        var controls = new Button { Text = "&Routing / Saver / Mode" };
        controls.SetBounds(20, 145, 240, 40);
        controls.Click += delegate { menu.Show(controls, new Point(0, controls.Height)); };
        window.Controls.Add(controls);
        var hide = new Button { Text = "&Hide to tray" };
        hide.SetBounds(280, 145, 160, 40);
        hide.Click += delegate { window.Hide(); };
        window.Controls.Add(hide);
        window.FormClosing += delegate(object sender, FormClosingEventArgs e) {
            if (!quitting && e.CloseReason == CloseReason.UserClosing) { e.Cancel = true; window.Hide(); }
        };

        status = Add("Starting...", null); status.Enabled = false;
        Add("Scope: acp-context-prompt only", null).Enabled = false;
        verification = Add("Last context request: none; connectivity unverified", null); verification.Enabled = false;
        menu.Items.Add(new ToolStripSeparator());
        routing = Add("&Routing enabled", delegate { Set("routing", routing.Checked ? "off" : "on"); });
        saver = Add("&Saver enabled", delegate { Set("saver", saver.Checked ? "off" : "on"); });
        mode = Add("&Mode", null);
        strict = new ToolStripMenuItem("&Strict", null, delegate { Set("mode", "strict"); });
        compatible = new ToolStripMenuItem("&Compatible", null, delegate { Set("mode", "compatible"); });
        mode.DropDownItems.AddRange(new ToolStripItem[] { strict, compatible });
        error = Add("", null); error.Enabled = false; error.Visible = false;
        menu.Items.Add(new ToolStripSeparator());
        Add("Open &history...", delegate { OpenHistory(); });
        Add("Open &controls window", delegate { ShowWindow(); });
        Add("&Quit tray (saved policy stays active)", delegate { quitting = true; ExitThread(); });
        menu.Opening += delegate { RefreshState(); };
        icon.ContextMenuStrip = menu;
        icon.MouseClick += delegate(object sender, MouseEventArgs e) {
            if (e.Button == MouseButtons.Left) menu.Show(Cursor.Position);
        };
        icon.DoubleClick += delegate { ShowWindow(); };
        try { Invoke("settings init"); } catch (Exception e) { lastError = e.Message; }
        RefreshState();
        icon.Visible = true;
        timer.Interval = 1500;
        timer.Tick += delegate {
            if (!menu.Visible && window.Enabled) RefreshState();
            var request = Path.Combine(home, "show-tray.request");
            if (File.Exists(request)) {
                try { File.Delete(request); ShowWindow(); } catch (IOException) { }
            }
        };
        timer.Start();
        ShowWindow();
    }

    ToolStripMenuItem Add(string text, EventHandler handler)
    {
        var item = new ToolStripMenuItem(text, null, handler);
        menu.Items.Add(item);
        return item;
    }

    string Run(string arguments)
    {
        var start = new ProcessStartInfo(executable, arguments) {
            UseShellExecute = false, CreateNoWindow = true, RedirectStandardOutput = true, RedirectStandardError = true
        };
        using (var process = Process.Start(start)) {
            var stdout = process.StandardOutput.ReadToEndAsync();
            var stderr = process.StandardError.ReadToEndAsync();
            if (!process.WaitForExit(5000)) { process.Kill(); throw new Exception("Tokenmill command timed out."); }
            if (process.ExitCode != 0) throw new Exception(stderr.Result.Trim());
            return stdout.Result;
        }
    }

    Dictionary<string, object> Invoke(string arguments)
    {
        return new JavaScriptSerializer().Deserialize<Dictionary<string, object>>(Run(arguments));
    }

    void Set(string key, string value)
    {
        try { Invoke("settings set " + key + " " + value); lastError = null; }
        catch (Exception e) { lastError = e.Message; }
        RefreshState();
    }

    void RefreshState()
    {
        try {
            var state = Invoke("settings show");
            if (!(bool)state["configured"]) throw new Exception("Settings missing. Restart tray to initialize safely.");
            routing.Enabled = true;
            routing.Checked = (bool)state["routing_enabled"];
            saver.Checked = (bool)state["saver_enabled"];
            saver.Enabled = mode.Enabled = routing.Checked;
            saver.Text = routing.Checked ? "&Saver enabled" : "&Saver enabled (inactive while routing OFF)";
            strict.Checked = (string)state["mode"] == "strict";
            compatible.Checked = !strict.Checked;
            bool running = (bool)state["running"];
            string next = routing.Checked ? "ON" : "OFF";
            status.Text = running ? "RUNNING | next request routing " + next : routing.Checked ? "READY | waiting for explicit request" : "Routing OFF | requests skipped";
            var last = state["last_request"] as Dictionary<string, object>;
            verification.Text = last == null ? "Last context request: none; connectivity unverified" : "Last context request: " + last["outcome"] + " / " + last["route_status"];
            if (!running && last != null && (string)last["outcome"] == "incomplete") status.Text = "ERROR | last request incomplete; next routing " + next;
            icon.Icon = routing.Checked ? SystemIcons.Information : SystemIcons.Application;
        } catch (Exception e) {
            lastError = e.Message;
            routing.Enabled = saver.Enabled = mode.Enabled = false;
            status.Text = "ERROR | policy unavailable";
        }
        error.Visible = lastError != null;
        if (lastError != null) {
            error.Text = "Last error: " + lastError;
            icon.Icon = SystemIcons.Warning;
            if (!status.Text.StartsWith("ERROR")) status.Text = "ERROR | " + status.Text;
        }
        summary.Text = status.Text;
        string tip = "Tokenmill: " + status.Text;
        icon.Text = tip.Substring(0, Math.Min(63, tip.Length));
    }

    void ShowWindow() { window.Show(); window.WindowState = FormWindowState.Normal; window.Activate(); }

    void OpenHistory()
    {
        using (var dialog = new OpenFileDialog { Title = "Open redacted Tokenmill evaluation history", Filter = "JSONL history (*.jsonl)|*.jsonl" }) {
            if (dialog.ShowDialog(window) != DialogResult.OK) return;
            try { MessageBox.Show(window, Run("eval-history \"" + dialog.FileName + "\""), "Tokenmill history"); }
            catch { MessageBox.Show(window, "History unavailable: invalid or unreadable redacted history.", "Tokenmill history"); }
        }
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing) { timer.Dispose(); icon.Visible = false; icon.Dispose(); menu.Dispose(); window.Dispose(); instance.Dispose(); }
        base.Dispose(disposing);
    }
}
