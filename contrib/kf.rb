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
  version "0.1.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-aarch64-macos.tar.gz"
      sha256 "20e14a98f3a5fe0cd18aa3e06442de6282a3a29e8a0d42f32c362bb0070c97d8"
    end
    on_intel do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-x86_64-macos.tar.gz"
      sha256 "982c91526c9433fb9952ea8c2566154c6880acb02e808fb7c99d1b0052a8dd9e"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-aarch64-linux.tar.gz"
      sha256 "539e5dbdf5df72887b964e32de6550b232c8ab6b7b2aa709f1d99eeb75030cce"
    end
    on_intel do
      url "https://github.com/syedazeez337/kubefuzz/releases/download/v#{version}/kf-x86_64-linux.tar.gz"
      sha256 "11bc48fe7f2101d327628a409d182d4a1db763b829c2290ae82e3e6ce98341a6"
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
