### File Structure

```
Header
|
|- Table Tree Root
|  |- Metadata Table [00000000-0000-0000-0000-000000000000]
|  |  |- Block Index [00000000-0000-0000-0000-000000000001]
|  |  |- Archetype Index [00000000-0000-0000-0000-000000000002]
|  |  '- Primary Key Index [00000000-0000-0000-0000-000000000003]
|  |- Table [Table 1 ID]
|  |  |- Block Index [00000000-0000-0000-0000-000000000001]
|  |  |  |- Data Block 1
|  |  |  |- Data Block 2
|  |  |  '- Data Block 3
|  |  |- Archetype Index [00000000-0000-0000-0000-000000000002]
|  |  '- Primary Key Index [00000000-0000-0000-0000-000000000003]
|  |     |- Snapshot 1, Row 1 -> [Data Block 2]
|  |     |- Snapshot 1, Row 2 -> [Data Block 3]
|  |     '- Snapshot 1, Row 3 -> [Data Block 1]
|  '- Table [Table 2 ID]
|     |- Block Index [00000000-0000-0000-0000-000000000001]
|     |- Archetype Index [00000000-0000-0000-0000-000000000002]
|     '- Primary Key Index [00000000-0000-0000-0000-000000000003]
|
'-Block Tree Root
    |- 0x0000_0000 -> [DB 1, Table 0, Index 0]
    |- 0x0000_0001 -> [DB 1, Table 1, Index 1]
    |- 0x0000_0002 -> [DB 1, Table 1, Index 1]
    |- 0x0000_0003 -> [DB 1, Table 2, Index 2]
    '- 0x0000_0004 -> [DB 1, Table 2, Index 2]

```

Table Tree Entry: (Table ID, Index ID, Addr)
Block Tree Entry: (Addr, Table ID, Index ID)

### Wire Protocol

gRPC over QUIC

### Limitations

Max. database size = 4 PB

Max. concurrent queries = 4096

Max. query temp space = 256 TB

### Backup & Recovery

Backups over FS