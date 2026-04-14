class Cxusage < Formula
  desc "Local TUI monitor for Codex usage"
  homepage "https://github.com/HayattiQ/codex-usage-monitor"
  url "https://github.com/HayattiQ/codex-usage-monitor/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "9233bca222e259413dffb724c97a401ad6a001604d1b0b2ec0a49ef6926a655c"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"cxusage", "doctor", "--codex-dir", testpath/".codex", "--data-dir", testpath/"data"
  end
end
