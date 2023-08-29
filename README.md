# Git-Pages 
**Serve and manage simple static sites using git!**

Git-Pages is a WIP standalone service for serving static websites using any git server you wish.

# Features
- Git agnostic: Use any git server you want
- Minimal
    - Built with minimal libraries
    - Do one thing: Serving static files! No HTTPS cert management, etc.
- Easy to use:
    - HTTP `GET` to get page
    - HTTP `PUT` to pull new page

# Quick-start
Release binaries soon(tm). For now, just clone this repository and build with Rust `cargo`

To run this app, you must pass the following environmental variables
- `ROOT_DOMAIN`: The root domain for pages (i.e. hyang.xyz)
- `GIT_DOMAIN`: The domain for the Git server. **MUST** include the protocol! (i.e. https://git.hyang.xyz)

Pages are served in a format similar to [Codeberg Pages](https://codeberg.page/):  
`https://[REPO].USERNAME.{DOMAIN}`  
Where *REPO* is optional, in which case by default it looks for the `pages` branch.

To clone/pull a page (using curl):
- `curl -X PUT https://[REPO].USERNAME.{DOMAIN}`
