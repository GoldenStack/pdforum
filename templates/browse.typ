#import "common.typ": *

#let info = yaml("info.yml")

#set page(columns: 2)

#show: common.with(info)

#set text(12pt)

#let post(id, author, likes, comments, liked, publish_time, content) = {
  let liked = liked == "true"
  
  link(info.url + "/user/" + author, strong(author))
  h(0.5em)
  text(size: 0.9em, fill: luma(100), smallcaps[Author])
  h(0.5em)
  sym.dot
  h(0.5em)
  smallcaps(text(fill: luma(100), publish_time))

  linebreak()
  pad(y: 0em, link(info.url + "/post/" + id, content))
  
  let like-link = info.url + if liked {
    "/unlike/"
  } else {
    "/like/"
  } + id + "?"
  
  box(stroke: none, height: auto, align(horizon, grid(
    columns: (auto, 2em, auto, 5fr),
    rows: (18pt),
    column-gutter: 3pt,
    link(like-link, image(if liked {"svg/filled-heart.svg"} else {"svg/heart.svg"})),
    link(like-link, likes),
    image("svg/comment.svg"),
    comments,
  )))

  line(length: 100%, stroke: luma(200))
  v(0.25em)

}

#{
  let data = read("data.txt").split("\u{0}")
  
  if data != ("", ) {
    for i in range(data.len(), step: 7) {
      post(data.at(i), data.at(i + 1), data.at(i + 2), data.at(i + 3), data.at(i + 4), data.at(i + 5), data.at(i + 6))
    }
  }
}