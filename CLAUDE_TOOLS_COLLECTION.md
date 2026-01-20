# Claude Code Tools, Plugins & MCP Servers Collection

## Awesome GitHub Listak (Bookmark ezeket!)

- [wong2/awesome-mcp-servers](https://github.com/wong2/awesome-mcp-servers) - A legnagyobb MCP szerver lista
- [punkpeye/awesome-mcp-servers](https://github.com/punkpeye/awesome-mcp-servers) - Kuratalt MCP lista
- [jmanhype/awesome-claude-code](https://github.com/jmanhype/awesome-claude-code) - Claude Code plugins, MCP, integrations
- [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers) - Hivatalos Anthropic MCP szerverek
- [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code) - Skills, hooks, slash-commands, agents

---

## TOP MCP Szerverek

### Development & GitHub
| Nev | Leiras |
|-----|--------|
| **GitHub MCP Server** | Repo kezeles, issues, PRs, CI/CD workflows |
| **Supabase MCP Server** | Hivatalos Supabase kapcsolat OAuth-tal |
| **Context7 MCP** | Naprakesz library dokumentacio injektalas |

### Database
| Nev | Leiras |
|-----|--------|
| **PostgreSQL MCP Server** | Termeszetes nyelvu DB lekerdezesek |
| **SQLite MCP** | SQLite kezeles |
| **Excel MCP Server** | Excel fajlok natural language kezeles |

### Browser Automation
| Nev | Leiras |
|-----|--------|
| **Playwright MCP Server** | Web automatizalas accessibility snapshot-okkal |
| **Puppeteer MCP** | Chrome automatizalas AI vision-nel |

### AI Integration
| Nev | Leiras |
|-----|--------|
| **Gemini MCP Server** | Google Gemini integracio - code review, brainstorm |
| **Sequential Thinking MCP** | Strukturalt gondolkodasi folyamat |

### Productivity & Design
| Nev | Leiras |
|-----|--------|
| **Notion MCP** | Hivatalos Notion integracio OAuth-tal |
| **Figma MCP Server** | Figma design struktura hozzaferes |
| **Zapier MCP Server** | Cross-app workflow automatizalas |

### Code & Memory
| Nev | Leiras |
|-----|--------|
| **Semantic Code Search MCP** | Hibrid kereses (BM25 + vector) 40% token csokkentes |
| **Memory Bank MCP Server** | Context megtartas session-ok kozott |

---

## Plugins & Extensions

### Marketplace-ek
- [buildwithclaude.com](https://www.buildwithclaude.com/) - 400+ extension
- [claudemarketplaces.com](https://claudemarketplaces.com/) - Plugin marketplace
- [Dev-GOM/claude-code-marketplace](https://github.com/Dev-GOM/claude-code-marketplace) - GitHub marketplace

### Top Plugins
| Nev | Leiras |
|-----|--------|
| **claudekit** | Auto-save checkpointing, 20+ subagent (code-reviewer, typescript-expert, stb.) |
| **ContextKit** | 4-fazisu tervezes, quality agents |
| **Claude Code Hook Comms (HCOM)** | Multi-agent kommunikacio hooks-szal |

### Editor Integrations
- **Claude Code for VS Code** - Hivatalos VS Code extension, inline diffs
- **Claude Code for JetBrains** - IntelliJ family plugin (Beta)

---

## Agent Rendszerek

### wshobson/agents Repository
- 100 specializalt AI agent
- 15 multi-agent workflow orchestrator
- 110 agent skill
- 76 development tool
- [GitHub Link](https://github.com/wshobson/agents)

### Claude Agent SDK
Hivatalos SDK autonom agent epiteshez - fajlok irasa, parancsok futtatasa, iterativ munka.

---

## Hasznos Linkek

- [MCP Tool Search](https://code.claude.com/docs) - Lazy loading, 95% context csokkentes
- [Claude Code Docs](https://code.claude.com/docs/en/plugins) - Hivatalos plugin dokumentacio
- [mcpcat.io](https://mcpcat.io/guides/best-mcp-servers-for-claude-code/) - MCP guide-ok
- [claudelog.com](https://claudelog.com/claude-code-mcps/) - Tutorials & best practices

---

## Telepites Tippek

### MCP Server telepites
```bash
# Peldak
claude mcp add github
claude mcp add postgres
claude mcp add playwright
```

### Plugin telepites
```bash
# Marketplace plugin
/plugin marketplace add ananddtyagi/claude-code-marketplace

# GitHub-rol
/plugin add https://github.com/username/plugin-name
```

---

## Best Practices

1. **Kezdd read-only szerverekkel** (docs, search, observability)
2. **Scope-olj minden szervert** - per-project keys, limited directories
3. **Loggolj mindent** - lasd hogyan hasznaljak az agent-ek a tool-okat
4. **Hasznald az MCP Tool Search-ot** - lazy loading a context megtakaritasert

---

*Osszegyujtve: 2026-01-20*
