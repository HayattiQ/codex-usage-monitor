class Cxusage < Formula
  desc "Local TUI monitor for Codex usage"
  homepage "https://github.com/HayattiQ/codex-usage-monitor"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/HayattiQ/codex-usage-monitor/releases/download/v0.1.0/cxusage-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    else
      url "https://github.com/HayattiQ/codex-usage-monitor/releases/download/v0.1.0/cxusage-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      odie "Linux ARM builds are not packaged yet"
    end

    url "https://github.com/HayattiQ/codex-usage-monitor/releases/download/v0.1.0/cxusage-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "REPLACE_WITH_SHA256"
  end

  def install
    bin.install "cxusage"
  end

  test do
    system "#{bin}/cxusage", "doctor", "--codex-dir", testpath/".codex", "--data-dir", testpath/"data"
  end
end
