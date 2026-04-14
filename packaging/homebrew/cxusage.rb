class Cxusage < Formula
  desc "Local TUI monitor for Codex usage"
  homepage "https://github.com/HayattiQ/codex-usage-monitor"
  url "https://github.com/HayattiQ/codex-usage-monitor/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "273654dc5eeb3a4365f44ade42d892bab4e107f2815d0b7a0aa40a62c25152ad"
  version "0.1.0"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/cxusage", "doctor", "--codex-dir", testpath/".codex", "--data-dir", testpath/"data"
  end
end
