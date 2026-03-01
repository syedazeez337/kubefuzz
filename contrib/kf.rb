# Homebrew formula for kf (KubeRift).
#
# To set up a tap:
#   brew tap syedazeez337/kuberift https://github.com/syedazeez337/kuberift
#   brew install kf
#
# Or install directly from this file:
#   brew install --formula contrib/kf.rb
#
# Update sha256 values after each release using:
#   shasum -a 256 kf-*.tar.gz

class Kf < Formula
  desc "Fuzzy-first interactive Kubernetes resource navigator"
  homepage "https://github.com/syedazeez337/kuberift"
  version "0.1.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/syedazeez337/kuberift/releases/download/v#{version}/kf-aarch64-macos.tar.gz"
      sha256 "87b01e2b6ec88fb52869d7364b36eb7350313cb04546f02c32466febc90d2470"
    end
    on_intel do
      url "https://github.com/syedazeez337/kuberift/releases/download/v#{version}/kf-x86_64-macos.tar.gz"
      sha256 "cf068ea8f6a65c29b9e8479c1fe5cb31dc24b75312fb7441e084ab61532ab114"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/syedazeez337/kuberift/releases/download/v#{version}/kf-aarch64-linux.tar.gz"
      sha256 "f68c11712fe93fe8079e67b00b6ec75113915a7eb96e3f9efe5e38c0ee69d8e2"
    end
    on_intel do
      url "https://github.com/syedazeez337/kuberift/releases/download/v#{version}/kf-x86_64-linux.tar.gz"
      sha256 "9df695135f3791976252f25e0d6fcdf0da1b973ab3f53ac816005a685e8dcbeb"
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
