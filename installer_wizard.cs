using System;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using System.Windows.Forms;
using Microsoft.Win32;

[assembly: AssemblyTitle("VN Surf Setup")]
[assembly: AssemblyProduct("VN Surf")]
[assembly: AssemblyVersion("1.2.2.0")]
[assembly: AssemblyFileVersion("1.2.2.0")]

namespace VNBrowserSetup
{
    public class InstallerForm : Form
    {
        [DllImport("user32.dll")]
        private static extern bool SetProcessDPIAware();

        [DllImport("user32.dll")]
        private static extern bool ReleaseCapture();

        [DllImport("user32.dll")]
        private static extern IntPtr SendMessage(IntPtr hWnd, int msg, int wParam, int lParam);

        private const int WM_NCLBUTTONDOWN = 0xA1;
        private const int HT_CAPTION = 0x2;

        private TextBox pathBox;
        private Button installBtn;
        private Button browseBtn;
        private CheckBox chkDesktop;
        private CheckBox chkStartMenu;
        private Label statusLbl;
        private ProgressBar progressBar;

        public InstallerForm()
        {
            float scale = 1.0f;
            using (var g = CreateGraphics()) { scale = g.DpiX / 96f; }
            Func<int, int> S = px => (int)Math.Round(px * scale);

            Text = "Cài đặt VN Surf v1.2.2";
            ClientSize = new Size(S(600), S(360));
            StartPosition = FormStartPosition.CenterScreen;
            FormBorderStyle = FormBorderStyle.None;
            BackColor = Color.FromArgb(15, 23, 42);
            ForeColor = Color.White;
            Font = new Font("Segoe UI", 9.5f);
            DoubleBuffered = true;

            try { Icon = Icon.ExtractAssociatedIcon(Application.ExecutablePath); } catch { }

            // Subtle outer border
            Paint += (s, e) =>
            {
                e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
                using (var pen = new Pen(Color.FromArgb(56, 189, 248), 1.5f))
                {
                    e.Graphics.DrawRectangle(pen, 0, 0, ClientSize.Width - 1, ClientSize.Height - 1);
                }
            };

            // Custom dark launcher title bar
            var headerBar = new Panel
            {
                Dock = DockStyle.Top,
                Height = S(44),
                BackColor = Color.FromArgb(11, 18, 34)
            };
            headerBar.MouseDown += (s, e) =>
            {
                if (e.Button == MouseButtons.Left)
                {
                    ReleaseCapture();
                    SendMessage(Handle, WM_NCLBUTTONDOWN, HT_CAPTION, 0);
                }
            };

            var iconBox = new PictureBox
            {
                Location = new Point(S(16), S(10)),
                Size = new Size(S(24), S(24)),
                SizeMode = PictureBoxSizeMode.Zoom,
                BackColor = Color.Transparent
            };
            try
            {
                if (Icon != null) iconBox.Image = Icon.ToBitmap();
            }
            catch { }
            headerBar.Controls.Add(iconBox);

            var hdrTitle = new Label
            {
                Text = "VN Surf Launcher Setup  •  v1.2.2",
                Font = new Font("Segoe UI", 10f, FontStyle.Bold),
                ForeColor = Color.FromArgb(226, 232, 240),
                Location = new Point(S(48), S(12)),
                AutoSize = true,
                UseMnemonic = false
            };
            hdrTitle.MouseDown += (s, e) =>
            {
                if (e.Button == MouseButtons.Left)
                {
                    ReleaseCapture();
                    SendMessage(Handle, WM_NCLBUTTONDOWN, HT_CAPTION, 0);
                }
            };
            headerBar.Controls.Add(hdrTitle);

            var closeBtn = new Button
            {
                Text = "✕",
                Size = new Size(S(44), S(44)),
                Location = new Point(ClientSize.Width - S(44), 0),
                FlatStyle = FlatStyle.Flat,
                BackColor = Color.Transparent,
                ForeColor = Color.FromArgb(148, 163, 184),
                Font = new Font("Segoe UI", 11f, FontStyle.Bold),
                Cursor = Cursors.Hand,
                UseMnemonic = false
            };
            closeBtn.FlatAppearance.BorderSize = 0;
            closeBtn.FlatAppearance.MouseOverBackColor = Color.FromArgb(220, 38, 38);
            closeBtn.MouseEnter += (s, e) => closeBtn.ForeColor = Color.White;
            closeBtn.MouseLeave += (s, e) => closeBtn.ForeColor = Color.FromArgb(148, 163, 184);
            closeBtn.Click += (s, e) => Application.Exit();
            headerBar.Controls.Add(closeBtn);
            Controls.Add(headerBar);

            // Hero banner
            var title = new Label
            {
                Text = "VN Surf — Trình duyệt nhẹ, màu sắc & bảo mật",
                Font = new Font("Segoe UI", 13.5f, FontStyle.Bold),
                Location = new Point(S(24), S(62)),
                MaximumSize = new Size(ClientSize.Width - S(48), 0),
                AutoSize = true,
                ForeColor = Color.FromArgb(52, 211, 153),
                UseMnemonic = false
            };
            Controls.Add(title);

            var sub = new Label
            {
                Text = "Chặn quảng cáo YouTube • Cửa sổ nổi PiP • Khoá mật khẩu DPAPI • Nhanh & Gọn",
                Font = new Font("Segoe UI", 9f),
                Location = new Point(S(26), S(96)),
                MaximumSize = new Size(ClientSize.Width - S(52), 0),
                AutoSize = true,
                ForeColor = Color.FromArgb(148, 163, 184),
                UseMnemonic = false
            };
            Controls.Add(sub);

            var lbl = new Label
            {
                Text = "Thư mục cài đặt:",
                Font = new Font("Segoe UI", 9.5f, FontStyle.Bold),
                Location = new Point(S(24), S(136)),
                AutoSize = true,
                ForeColor = Color.FromArgb(226, 232, 240),
                UseMnemonic = false
            };
            Controls.Add(lbl);

            string defDir = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "Programs", "VN Surf"
            );
            pathBox = new TextBox
            {
                Text = defDir,
                Location = new Point(S(26), S(162)),
                Width = S(448),
                Font = new Font("Segoe UI", 9.5f),
                BackColor = Color.FromArgb(30, 41, 59),
                ForeColor = Color.White,
                BorderStyle = BorderStyle.FixedSingle
            };
            Controls.Add(pathBox);

            browseBtn = new Button
            {
                Text = "Chọn...",
                Location = new Point(S(484), S(160)),
                Width = S(90),
                Height = S(29),
                FlatStyle = FlatStyle.Flat,
                BackColor = Color.FromArgb(51, 65, 85),
                ForeColor = Color.White,
                Cursor = Cursors.Hand,
                UseMnemonic = false
            };
            browseBtn.FlatAppearance.BorderColor = Color.FromArgb(71, 85, 105);
            browseBtn.Click += (s, e) =>
            {
                using (var fbd = new FolderBrowserDialog())
                {
                    if (fbd.ShowDialog() == DialogResult.OK)
                        pathBox.Text = Path.Combine(fbd.SelectedPath, "VN Surf");
                }
            };
            Controls.Add(browseBtn);

            chkDesktop = new CheckBox
            {
                Text = "Tạo lối tắt ngoài Desktop",
                Checked = true,
                Location = new Point(S(26), S(206)),
                AutoSize = true,
                ForeColor = Color.FromArgb(203, 213, 225),
                UseMnemonic = false
            };
            Controls.Add(chkDesktop);

            chkStartMenu = new CheckBox
            {
                Text = "Thêm vào Start Menu",
                Checked = true,
                Location = new Point(S(240), S(206)),
                AutoSize = true,
                ForeColor = Color.FromArgb(203, 213, 225),
                UseMnemonic = false
            };
            Controls.Add(chkStartMenu);

            progressBar = new ProgressBar
            {
                Location = new Point(S(26), S(246)),
                Width = ClientSize.Width - S(52),
                Height = S(10),
                Minimum = 0,
                Maximum = 100,
                Value = 0,
                Style = ProgressBarStyle.Continuous
            };
            Controls.Add(progressBar);

            statusLbl = new Label
            {
                Text = "Sẵn sàng cài đặt VN Surf lên máy tính của bạn.",
                Font = new Font("Segoe UI", 9f),
                Location = new Point(S(24), S(268)),
                MaximumSize = new Size(ClientSize.Width - S(48), 0),
                AutoSize = true,
                ForeColor = Color.FromArgb(148, 163, 184),
                UseMnemonic = false
            };
            Controls.Add(statusLbl);

            installBtn = new Button
            {
                Text = "Cài đặt & Mở ngay",
                Location = new Point(ClientSize.Width - S(216), S(300)),
                Width = S(190),
                Height = S(40),
                FlatStyle = FlatStyle.Flat,
                BackColor = Color.FromArgb(16, 185, 129),
                ForeColor = Color.White,
                Font = new Font("Segoe UI", 10f, FontStyle.Bold),
                Cursor = Cursors.Hand,
                UseMnemonic = false
            };
            installBtn.FlatAppearance.BorderSize = 0;
            installBtn.Click += async (s, e) => await StartInstallAsync();
            Controls.Add(installBtn);
        }

        private async Task StartInstallAsync()
        {
            installBtn.Enabled = false;
            browseBtn.Enabled = false;
            pathBox.Enabled = false;
            string targetDir = pathBox.Text.Trim();
            bool mkDesktop = chkDesktop.Checked;
            bool mkStart = chkStartMenu.Checked;

            statusLbl.Text = "Đang giải nén trình duyệt VN Surf...";
            progressBar.Value = 25;
            await Task.Delay(120);

            string exePath = null;
            await Task.Run(() =>
            {
                exePath = DoInstall(targetDir, mkDesktop, mkStart, false);
            });

            progressBar.Value = 75;
            statusLbl.Text = "Đang tạo lối tắt & hoàn tất cấu hình...";
            await Task.Delay(180);

            progressBar.Value = 100;
            statusLbl.Text = "Cài đặt hoàn tất! Đang khởi chạy VN Surf...";
            await Task.Delay(220);

            if (!string.IsNullOrEmpty(exePath) && File.Exists(exePath))
            {
                Process.Start(exePath);
            }
            Application.Exit();
        }

        public static string DoInstall(string targetDir, bool mkDesktop, bool mkStart, bool autoLaunch)
        {
            foreach (var name in new[] { "VN Surf", "VN Browser", "vn-browser" })
            {
                foreach (var p in Process.GetProcessesByName(name))
                {
                    try { p.Kill(); p.WaitForExit(1500); } catch { }
                }
            }

            Directory.CreateDirectory(targetDir);
            string exePath = Path.Combine(targetDir, "VN Surf.exe");
            using (var res = Assembly.GetExecutingAssembly().GetManifestResourceStream("VNBrowser.exe"))
            using (var fs = new FileStream(exePath, FileMode.Create, FileAccess.Write))
            {
                res.CopyTo(fs);
            }

            string startMenu = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.StartMenu),
                "Programs", "VN Surf.lnk"
            );
            string desktop = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory),
                "VN Surf.lnk"
            );

            string targets = "";
            if (mkStart) targets += "'" + startMenu.Replace("'", "''") + "'";
            if (mkDesktop)
            {
                if (targets.Length > 0) targets += ",";
                targets += "'" + desktop.Replace("'", "''") + "'";
            }
            if (targets.Length > 0)
            {
                string ps = string.Format(
                    "$w = New-Object -ComObject WScript.Shell; " +
                    "foreach ($p in @({0})) {{ $s = $w.CreateShortcut($p); $s.TargetPath = '{1}'; $s.WorkingDirectory = '{2}'; $s.IconLocation = '{1},0'; $s.Save(); }}",
                    targets,
                    exePath.Replace("'", "''"),
                    targetDir.Replace("'", "''")
                );
                var psi = new ProcessStartInfo("powershell", "-NoProfile -WindowStyle Hidden -Command \"" + ps + "\"")
                {
                    CreateNoWindow = true,
                    UseShellExecute = false
                };
                try { Process.Start(psi).WaitForExit(5000); } catch { }
            }

            try
            {
                using (var k = Registry.CurrentUser.CreateSubKey(@"Software\Microsoft\Windows\CurrentVersion\Uninstall\VNBrowser"))
                {
                    if (k != null)
                    {
                        k.SetValue("DisplayName", "VN Surf");
                        k.SetValue("DisplayIcon", exePath + ",0");
                        k.SetValue("DisplayVersion", "1.2.2");
                        k.SetValue("InstallLocation", targetDir);
                    }
                }
            }
            catch { }

            if (autoLaunch)
            {
                Process.Start(exePath);
            }
            return exePath;
        }

        [STAThread]
        static void Main(string[] args)
        {
            try { SetProcessDPIAware(); } catch { }
            if (args.Length >= 2 && args[0].Equals("/silent", StringComparison.OrdinalIgnoreCase))
            {
                DoInstall(args[1], true, true, true);
                return;
            }
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Application.Run(new InstallerForm());
        }
    }
}
