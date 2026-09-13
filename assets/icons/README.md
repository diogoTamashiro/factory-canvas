# Icon assets

This directory holds optional local PNG icons for catalog buildables.

- One dedicated, versioned directory inside the project — not a
  separate Git repository or submodule.
- A buildable's catalog JSON entry may reference a file here through an
  optional `icon` field, using a path relative to this directory (for
  example `"icon": "xiranite_power_pole.png"`).
- Only static PNG is supported. A missing, unreadable, or unsupported
  file falls back to the buildable's existing abbreviated text — it
  never blocks the catalog from loading.
- Changes here apply after an application restart; a
  `catalog/public/`-embedded reference also needs a rebuild, exactly
  like the rest of the public catalog.

Full usage and troubleshooting instructions live in the project
`README.md`'s runtime-catalog section.
