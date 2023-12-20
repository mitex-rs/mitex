
#import "prelude.typ": *
#include "standard.typ"

#locate(loc => {
  let packages = packages-all(loc);
  [
    #metadata(packages) <mitex-packages>
    #packages
  ]
})
