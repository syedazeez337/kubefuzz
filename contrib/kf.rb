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
      sha256 "b4f0dca348998b5e83f9a25597008f656a549a4c8a003c52780ede536d518759"
    end
    on_intel do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-x86_64-macos.tar.gz"
      sha256 "ac072730787f6aad607248002ae3e6d091804649c95bed76ed0c985d2ef53ff2"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-aarch64-linux.tar.gz"
      sha256 "b18c666a02c1cc5c73f3fcf5f81abab974fd9c5032de5d83c994bb053b095c33"
    end
    on_intel do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-x86_64-linux.tar.gz"
      sha256 "3a9456b411db3d616d27f8e1ed0aa357d3918984506c6583b2e2c397bdd5a637"
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
