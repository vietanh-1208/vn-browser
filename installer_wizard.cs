using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Reflection;
using System.Windows.Forms;
using Microsoft.Win32;

[assembly: AssemblyTitle("VN Browser Setup")]
[assembly: AssemblyProduct("VN Browser")]
[assembly: AssemblyVersion("1.1.0.0")]
[assembly: AssemblyFileVersion("1.1.0.0")]

namespace VNBrowserSetup
{
    public class InstallerForm : Form
    {
        private TextBox pathBox;
        private Button installBtn;
        private Label statusLbl;

        public InstallerForm()
        {
            Text = "Cài đặt VN Browser v1.1.0";
            Size = new Size(480, 240);
            StartPosition = FormStartPosition.CenterScreen;
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            BackColor = Color.FromArgb(15, 23, 42);
            ForeColor = Color.White;

            var title = new Label
            {
                Text = "VN Browser - Trình duyệt nhẹ & bảo mật",
                Font = new Font("Segoe UI", 13, FontStyle.Bold),
                Location = new Point(22, 20),
                AutoSize = true,
                ForeColor = Color.FromArgb(34, 197, 94)
            };
            Controls.Add(title);

            var lbl = new Label
            {
                Text = "Thư mục cài đặt:",
                Font = new Font("Segoe UI", 9.5f),
                Location = new Point(22, 62),
                AutoSize = true
            };
            Controls.Add(lbl);

            string defDir = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "Programs", "VN Browser"
            );
            pathBox = new TextBox
            {
                Text = defDir,
                Location = new Point(24, 88),
                Width = 330,
                BackColor = Color.FromArgb(30, 41, 59),
                ForeColor = Color.White,
                BorderStyle = BorderStyle.FixedSingle
            };
            Controls.Add(pathBox);

            var browseBtn = new Button
            {
                Text = "Chọn...",
                Location = new Point(364, 86),
                Width = 80,
                Height = 26,
                FlatStyle = FlatStyle.Flat,
                BackColor = Color.FromArgb(51, 65, 85),
                ForeColor = Color.White
            };
            browseBtn.Click += (s, e) =>
            {
                using (var fbd = new FolderBrowserDialog())
                {
                    if (fbd.ShowDialog() == DialogResult.OK)
                        pathBox.Text = Path.Combine(fbd.SelectedPath, "VN Browser");
                }
            };
            Controls.Add(browseBtn);

            statusLbl = new Label
            {
                Text = "Sẵn sàng cài đặt VN Browser kèm lối tắt Desktop & Start Menu.",
                Font = new Font("Segoe UI", 9f),
                Location = new Point(22, 126),
                AutoSize = true,
                ForeColor = Color.LightGray
            };
            Controls.Add(statusLbl);

            installBtn = new Button
            {
                Text = "Cài đặt & Mở ngay",
                Location = new Point(280, 158),
                Width = 164,
                Height = 34,
                FlatStyle = FlatStyle.Flat,
                BackColor = Color.FromArgb(22, 163, 74),
                ForeColor = Color.White,
                Font = new Font("Segoe UI", 9.5f, FontStyle.Bold)
            };
            installBtn.Click += (s, e) => DoInstall(pathBox.Text);
            Controls.Add(installBtn);
        }

        public static void DoInstall(string targetDir)
        {
            foreach (var p in Process.GetProcessesByName("VN Browser"))
            {
                try { p.Kill(); p.WaitForExit(1500); } catch { }
            }
            Directory.CreateDirectory(targetDir);
            string exePath = Path.Combine(targetDir, "VN Browser.exe");
            using (var res = Assembly.GetExecutingAssembly().GetManifestResourceStream("VNBrowser.exe"))
            using (var fs = new FileStream(exePath, FileMode.Create, FileAccess.Write))
            {
                res.CopyTo(fs);
            }

            string startMenu = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.StartMenu),
                "Programs", "VN Browser.lnk"
            );
            string desktop = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory),
                "VN Browser.lnk"
            );
            string ps = string.Format(
                "$w = New-Object -ComObject WScript.Shell; " +
                "foreach ($p in @('{0}', '{1}')) {{ $s = $w.CreateShortcut($p); $s.TargetPath = '{2}'; $s.WorkingDirectory = '{3}'; $s.IconLocation = '{2},0'; $s.Save(); }}",
                startMenu.Replace("'", "''"),
                desktop.Replace("'", "''"),
                exePath.Replace("'", "''"),
                targetDir.Replace("'", "''")
            );
            var psi = new ProcessStartInfo("powershell", "-NoProfile -WindowStyle Hidden -Command \"" + ps + "\"")
            {
                CreateNoWindow = true,
                UseShellExecute = false
            };
            try { Process.Start(psi).WaitForExit(5000); } catch { }

            try
            {
                using (var k = Registry.CurrentUser.CreateSubKey(@"Software\Microsoft\Windows\CurrentVersion\Uninstall\VNBrowser"))
                {
                    if (k != null)
                    {
                        k.SetValue("DisplayName", "VN Browser");
                        k.SetValue("DisplayIcon", exePath + ",0");
                        k.SetValue("DisplayVersion", "1.1.0");
                        k.SetValue("InstallLocation", targetDir);
                    }
                }
            }
            catch { }

            Process.Start(exePath);
            Application.Exit();
        }

        [STAThread]
        static void Main(string[] args)
        {
            if (args.Length >= 2 && args[0].Equals("/silent", StringComparison.OrdinalIgnoreCase))
            {
                DoInstall(args[1]);
                return;
            }
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Application.Run(new InstallerForm());
        }
    }
}
