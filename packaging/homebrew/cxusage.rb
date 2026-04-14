class Cxusage < Formula
  desc "Local TUI monitor for Codex usage"
  homepage "https://github.com/HayattiQ/codex-usage-monitor"
  url "https://github.com/HayattiQ/codex-usage-monitor/archive/refs/tags/v0.1.2.tar.gz"
  sha256 "fec6bac730abd44c18b3ec37544fb9e7393cf2ac1e3a03b6043f67a5d3d5dd5d"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"cxusage", "doctor", "--codex-dir", testpath/".codex", "--data-dir", testpath/"data"
  end
end
