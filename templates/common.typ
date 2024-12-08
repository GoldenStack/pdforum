#let navbar(url, signed-in) = box(width: 100%, inset: 0.2em, stroke: black)[
    #box(width: 100%, inset: 0.35em, stroke: black)[
    #align(center)[
      #show link: underline
      
      #smallcaps(lower(link(url + "/")[BROWSE] + h(5%) + link(url + "/publish")[PUBLISH] + h(5%) + link(url + "/announcements")[ANNOUNCEMENTS] + h(5%) + {
        if signed-in {
          link(url + "/logout")[ACCOUNT] + h(5%) + link(url + "/settings")[SETTINGS]
        } else {
          link(url + "/login")[LOGIN] + h(5%) + link(url + "/register")[REGISTER]
        }
      }))
    ]
    #v(0.15em)
  ]
]

#let brand = grid(
  columns: (25%, 1fr, 25%),
  align(center)[
    #text(size: 2.3em)[
      #par(leading: 0.2em)[
        HTML-Free!
      ]
    ]
  ],
  align(center)[
    #text(size: 5em)[
      PDForum
    ]
  ],
  align(center + horizon)[
    #box(stroke: black, inset: 0.4em, width: 50%, outset: -5%)[
      #par(leading: 0.2em)[
        #show link: underline
        
        #smallcaps(link("https://github.com/goldenstack/pdforum/")[source])

        #v(-0.8em)
        
        #smallcaps[made by #link("https://goldenstack.net/")[golden]]

        #v(0.1em)
      ]
    ]
  ]
)

#let info = box(width: 100%, inset: 0.25em, stroke: (top: black, bottom: black))[
  #box(width: 100%, inset: 0.5em, stroke: (top: black, bottom: black))[
    #box(width: 1fr)[
      #align(left)[
        #h(2em)
        #smallcaps("web a-1a")
      ]
    ]
    #box(width: 1fr)[   
      #align(center)[
        #smallcaps(datetime.today().display("[weekday], [month repr:long] [day padding:none], [year]"))
      ]
    ]
    #box(width: 1fr)[
      #align(right)[
        #smallcaps("pdf edition")
        #h(2em)
      ] 
    ]
  ]
]

#let header(url, signed-in) = navbar(url, signed-in) + brand + info

#let footer = context [
  #set text(size: 8pt)
  #line(length: 100%,stroke: 0.2pt + gray)
  PDForum - #smallcaps(datetime.today().display("[year]-[month]-[day]"))
  #h(1fr)
  #counter(page).display(
    "1/1",
    both: true,
  )
]

#let common(info, doc) = {
  set text(size: 10pt, weight: "regular", style: "normal")
  
  set page("a4",
    margin:(top: 2cm,
    bottom: 2cm,
    left: 2.5cm, right: 2cm))
    
  set page(footer: footer)
  
  place(top, float: true, scope: "parent")[
    #show: header(info.url, info.auth)
  ]

  doc
}
