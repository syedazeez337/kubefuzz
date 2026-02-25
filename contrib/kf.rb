# Homebrew formula for kf (KubeFuzz).
#
# To set up a tap:
#   brew tap syedazeez337/kubefuzz https://github.com/syedazeez337/kubefuzz
#   brew install kf
#
# Or install directly from this file:
#   brew install --formula contrib/kf.rb
#
# Update sha256 values after each release using:
#   shasum -a 256 kf-*.tar.gz

class Kf < Formula
  desc "Fuzzy-first interactive Kubernetes resource navigator"
  homepage "https://github.com/syedazeez337/kubefuzz"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-aarch64-macos.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_MACOS"
    end
    on_intel do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-x86_64-macos.tar.gz"
      sha256 "PLACEHOLDER_X86_64_MACOS"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-aarch64-linux.tar.gz"
      sha256 "PLACEHOLDER_AARCH64_LINUX"
    end
    on_intel do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-x86_64-linux.tar.gz"
      sha256 "PLACEHOLDER_X86_64_LINUX"
    end
  end

  # kubectl is used at runtime for all actions (describe, logs, exec, delete…)
  depends_on "kubernetes-cli" => :recommended

  def install
    bin.install "kf"
    bash_completion.install "completions/kf.bash"
    zsh_completion.install  "completions/_kf"
    fish_completion.install "completions/kf.fish"
    man1.install "man/kf.1"
  end

  test do
    assert_match "kf #{version}", shell_output("#{bin}/kf --version")
  end
end
