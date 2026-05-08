#let render(
  title: "",
  date: "",
  read_time: 0,
  tags: (),
  description: "",
  cover_image: none,
  body: [],
) = {
  set page(
    paper: "a4",
    margin: (top: 2cm, bottom: 2.5cm, left: 2cm, right: 2cm),
    header: context {
      let page_num = counter(page).get().first()
      if page_num > 1 {
        set text(size: 7pt, fill: rgb("#999999"))
        grid(
          columns: (1fr, 1fr),
          align(left)[#title],
          align(right)[blog.bythewood.me],
        )
      }
    },
    footer: context {
      set align(center)
      set text(size: 7pt, fill: rgb("#999999"))
      [#counter(page).display() / #counter(page).final().first()]
    },
  )

  set text(
    font: ("Inter", "DejaVu Sans", "Liberation Sans", "Arial"),
    size: 10pt,
    fill: rgb("#1a1a1a"),
  )
  set par(leading: 0.65em, justify: false)

  show raw: set text(
    font: ("JetBrains Mono", "DejaVu Sans Mono", "Liberation Mono"),
    size: 8.5pt,
  )
  show raw.where(block: true): it => block(
    fill: rgb("#f5f5f5"),
    inset: 8pt,
    radius: 4pt,
    width: 100%,
    breakable: true,
    it,
  )
  show raw.where(block: false): it => box(
    fill: rgb("#f0f0f0"),
    inset: (x: 3pt, y: 1pt),
    radius: 3pt,
    outset: (y: 2pt),
    it,
  )

  show link: set text(fill: rgb("#0e3ff4"))
  show link: underline

  show heading: set block(above: 1.4em, below: 0.6em)
  show heading.where(level: 1): set text(size: 1.8em, weight: "bold")
  show heading.where(level: 2): set text(size: 1.4em, weight: "bold")
  show heading.where(level: 3): set text(size: 1.15em, weight: "bold")
  show heading.where(level: 4): set text(size: 1.0em, weight: "bold")

  show quote.where(block: true): it => block(
    inset: (left: 1em),
    stroke: (left: 3pt + rgb("#dddddd")),
  )[#set text(fill: rgb("#555555")); #it.body]

  // Title
  text(size: 1.8em, weight: "bold")[#title]

  // Meta line + tags + bottom border
  block(
    above: 0.4em,
    below: 1.5em,
    stroke: (bottom: 2pt + rgb("#eeeeee")),
    inset: (bottom: 0.8em),
    width: 100%,
  )[
    #set text(size: 0.9em, fill: rgb("#666666"))
    Isaac Bythewood · #date · #read_time min read
    #if tags.len() > 0 {
      v(0.4em)
      for tag in tags {
        box(
          fill: rgb("#eeeeee"),
          inset: (x: 0.5em, y: 0.1em),
          radius: 3pt,
          outset: (y: 1pt),
        )[#text(size: 0.85em)[#tag]]
        h(0.3em)
      }
    }
  ]

  if cover_image != none {
    align(center)[#image(cover_image, width: 100%)]
    v(1.5em)
  }

  if description != "" {
    block(below: 1.5em)[
      #set text(size: 1.1em, fill: rgb("#555555"))
      #description
    ]
  }

  body
}
