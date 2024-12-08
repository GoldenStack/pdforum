#import "common.typ": *

#let info = yaml("info.yml")

#set page(columns: 2)

#show: common.with(info)

#let post(author: none, id: none, time: none, content: none) = link(info.url + "/post/" + id, strong(author)) + h(0.5em) + text(size: 9pt, fill: luma(100), smallcaps[Author]) + h(0.5em) + sym.dot + h(0.5em) + smallcaps(text(fill: luma(100), link(info.url + "/post/" + id, time))) + v(0.25em) + content + v(1em)

#{
  let data = read("data.txt").split("\n")
  
  for i in range(data.len(), step: 4) {
    post(author: data.at(i), id: data.at(i + 1), time: data.at(i + 2), content: data.at(i + 3))
  }
}
