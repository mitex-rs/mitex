
#import "bencher.typ": *

#let data = json("/local/oiwiki-231222.json");

#show: integrate-conversion.with(data: data, convert-only: true)
