
#import "prelude.typ": *
#import "latex/standard.typ": package as latex-std

#let packages = (latex-std,)
#let mitex-scope = packages.map(pkg => pkg.scope).sum()

[
  #metadata(packages) <mitex-packages>
  #packages
]
