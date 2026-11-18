# mcp

An MCP server already puts text in front of the agent, so a skill over it holds only what that text cannot.

| text | in context | holds |
|---|---|---|
| tool name, description, input schema | whenever the tools are listed; some clients only after a tool search | what one tool does and takes |
| server `instructions` | every session the server is connected | what every session needs across tools: call order, a mandatory setup call, which skill to load before which tool |
| skill | when it fires | workflows across calls, domain reasoning, anything long |

- The skill never restates schemas or tool descriptions; the agent sees them. It names tools by the server's own
  tool name, since clients prefix them differently, and naming them is what lets a client that defers tool
  schemas find them.
- A rule that must hold before a tool is called goes in the tool description or the server `instructions`: a
  skill that has not fired cannot gate anything.
- The skill says what the task needs from the server, so an agent whose server failed to connect reports that
  rather than working around it with other tools.
- Tool names and arguments change with the server. A skill over our own server ships from its repo and releases
  with it (install and paths as in [repo.md](repo.md)); one over a third-party server states the server version
  it was written against in `compatibility`.

Testing runs with the server connected; the transcript shows which tools were called and in what order, the
first place a skill over tools goes wrong.
