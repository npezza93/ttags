{ pkgs }:
{
  greet = name: "hello ${name}";
  package = pkgs.callPackage ./package.nix {};
  ignored = 1;
}
