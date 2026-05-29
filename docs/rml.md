# RML Widget Gallery

This file is a complete RML gallery for the currently documented RML surface. It is meant to be used as:

- documentation for all supported tags
- a copy/paste example library
- a parser/runtime smoke-test checklist
- a starting point for `examples/rml_widget_gallery.rs`

RML is rgui's XML-like declarative syntax. It maps directly to `rgui::Element` and the widget builder API.

Enable RML with:

```toml
[dependencies]
rgui = { path = ".", features = ["rml"] }
```

Parse a document:

```rust
let parsed = rgui::rml::parse(include_str!("rml_widget_gallery.rml"))?;
let root = parsed.element;

for warning in parsed.warnings {
    eprintln!("RML warning: {}", warning.message);
}
```

---

# Supported Gallery Tags

## Layout

- `Row`
- `Column`
- `Grid`
- `Stack`
- `Absolute`
- `ScrollArea`
- `Text`
- `Canvas`

## Forms

- `Button`
- `TextInput`
- `Input`
- `Checkbox`
- `Radio`
- `Select`
- `Option`
- `SelectOption`
- `Textarea`

## Collections

- `Tabs`
- `Tab`
- `List`
- `ListItem`
- `Table`
- `Columns`
- `ColumnDef`
- `TableRow`
- table-local `Row`
- `Cell`
- `Tree`
- `TreeItem`
- `Menu`
- `MenuItem`
- `ContextMenu`

## Overlays

- `Popover`
- `Tooltip`
- `Modal`

## Primitive Widgets

- `Icon`
- `Divider`

---

# Gallery Runtime Shell

Use a scrollable root so the gallery works in small windows:

```xml
<ScrollArea key="gallery-root" width="100%" height="100%" background="#ffffff">
  <Column key="gallery-page" width="920" padding="16" gap="16">
    <Text key="gallery-title" heading>RML Widget Gallery</Text>
  </Column>
</ScrollArea>
```

---

# 1. Layout Gallery

## 1.1 Column

```xml
<Column key="column-demo" padding="12" gap="8" background="#ffffff" radius="8">
  <Text heading>Column</Text>
  <Text>Children are stacked vertically.</Text>
  <Button key="column-button">Button inside column</Button>
</Column>
```

## 1.2 Row

```xml
<Row key="row-demo" gap="8" align-items="center" background="#ffffff" padding="12" radius="8">
  <Icon name="search" />
  <TextInput key="row-search" placeholder="Search" />
  <Button key="row-submit" primary>Search</Button>
</Row>
```

## 1.3 Grid

```xml
<Grid key="grid-demo" grid-template-columns="1fr 1fr" gap="8" background="#ffffff" padding="12" radius="8">
  <Text background="#eef2ff" padding="8">Cell 1</Text>
  <Text background="#ecfeff" padding="8">Cell 2</Text>
  <Text background="#fef9c3" padding="8">Cell 3</Text>
  <Text background="#fce7f3" padding="8">Cell 4</Text>
</Grid>
```

## 1.4 Stack

```xml
<Stack key="stack-demo" width="240" height="120" background="#ffffff" radius="8">
  <Text padding="12">Stack base layer</Text>
  <Button key="stack-floating" width="120" z-index="2">Floating</Button>
</Stack>
```

## 1.5 Absolute

```xml
<Absolute key="absolute-demo" width="260" height="140" background="#ffffff" radius="8">
  <Text padding="8">Absolute area</Text>
  <Button key="absolute-top-left" position="absolute" inset="40 auto auto 12">Top Left</Button>
  <Button key="absolute-bottom-right" position="absolute" inset="auto 12 12 auto">Bottom Right</Button>
</Absolute>
```

## 1.6 ScrollArea

`axis` may be `x`, `y`, or `both`. If omitted, both axes are scrollable.

```xml
<ScrollArea key="scroll-demo" height="120" axis="y" background="#ffffff" radius="8">
  <Column gap="4" padding="8">
    <Text height="32">Scrollable line 1</Text>
    <Text height="32">Scrollable line 2</Text>
    <Text height="32">Scrollable line 3</Text>
    <Text height="32">Scrollable line 4</Text>
    <Text height="32">Scrollable line 5</Text>
  </Column>
</ScrollArea>
```

## 1.7 Text

```xml
<Column key="text-demo" gap="4" background="#ffffff" padding="12" radius="8">
  <Text heading>Text</Text>
  <Text font-size="16" font-weight="normal">Body text</Text>
  <Text color="#2563eb" font-weight="bold">Colored bold text</Text>
</Column>
```

## 1.8 Canvas

`Canvas` currently identifies a named drawing surface. It is useful as a placeholder until the custom drawing API is finalized.

```xml
<Canvas key="canvas-demo" name="chart" width="240" height="140" background="#ffffff" />
```

---

# 2. Forms Gallery

## 2.1 Button

```xml
<Row key="button-demo" gap="8" align-items="center" flex-wrap="wrap">
  <Button key="button-primary" primary on-click="save">Primary</Button>
  <Button key="button-secondary" variant="secondary">Secondary</Button>
  <Button key="button-disabled" disabled>Disabled</Button>
  <Button key="button-loading" loading>Loading</Button>
</Row>
```

## 2.2 TextInput and Input

`Input` is an alias for `TextInput`.

```xml
<Column key="input-demo" gap="8">
  <TextInput key="name-input" placeholder="Name" default-value="Alice" aria-label="Name" />
  <Input key="email-input" placeholder="Email" default-value="alice@example.com" />
  <TextInput key="password-input" placeholder="Password" password />
</Column>
```

## 2.3 Checkbox

```xml
<Column key="checkbox-demo" gap="8">
  <Checkbox key="enabled-checkbox" checked label="Enabled" />
  <Checkbox key="terms-checkbox">Accept terms</Checkbox>
  <Checkbox key="indeterminate-checkbox" indeterminate label="Partially selected" />
</Column>
```

## 2.4 Radio

Radio exists, but a future `RadioGroup` should own exclusive selection behavior.

```xml
<Column key="radio-demo" gap="8">
  <Radio key="radio-free" value="free" label="Free" checked />
  <Radio key="radio-pro" value="pro" label="Pro" />
  <Radio key="radio-enterprise" value="enterprise" label="Enterprise" disabled />
</Column>
```

Recommended future syntax:

```xml
<RadioGroup key="plan" value="pro">
  <Radio value="free">Free</Radio>
  <Radio value="pro">Pro</Radio>
</RadioGroup>
```

## 2.5 Select, Option, SelectOption

Use `SelectStyle` children to configure select part styles. Supported parts are `trigger`, `popover`, `list`, `item`, `item-hovered`, `item-selected`, and `item-disabled`.

```xml
<Row key="select-row" gap="8" flex-wrap="wrap">
  <Select key="priority-select" placeholder="Priority" default-value="medium">
    <SelectStyle part="trigger" height="32" />
    <SelectStyle part="popover" width="220" />
    <SelectStyle part="item" padding="8" />
    <Option value="low">Low</Option>
    <Option value="medium">Medium</Option>
    <Option value="high" disabled>High</Option>
  </Select>

  <Select key="size-select" placeholder="Size">
    <SelectOption value="s">Small</SelectOption>
    <SelectOption value="m">Medium</SelectOption>
    <SelectOption value="l">Large</SelectOption>
  </Select>
</Row>
```

## 2.6 Textarea

```xml
<Textarea key="notes-textarea" placeholder="Notes" default-value="Initial notes" rows="4" />
```

---

# 3. Collections Gallery

## 3.1 Tabs and Tab

```xml
<Tabs key="settings-tabs" active-index="0">
  <Tab label="General" />
  <Tab label="Advanced" />
  <Tab label="About" />
</Tabs>
```

Compact syntax:

```xml
<Tabs key="compact-tabs" tabs="Overview,Details,Activity" active-index="1" />
```

## 3.2 List and ListItem

```xml
<List key="task-list" selected-index="1">
  <ListItem>Inbox</ListItem>
  <ListItem>Today</ListItem>
  <ListItem>Done</ListItem>
</List>
```

Compact syntax:

```xml
<List key="compact-list" items="Inbox,Today,Done" selected-index="0" />
```

## 3.3 Table with TableRow

```xml
<Table key="status-table" columns="Name,Status" selected-row="0">
  <TableRow values="Runtime,Ready" />
  <TableRow values="Renderer,Ready" />
  <TableRow values="RML,Ready" />
</Table>
```

## 3.4 Table with Columns, ColumnDef, Row, and Cell

Inside `Table`, `Row` is table-local row syntax.

```xml
<Table key="detailed-table" selected-row="1">
  <Columns>
    <ColumnDef>Name</ColumnDef>
    <ColumnDef>Status</ColumnDef>
    <ColumnDef>Owner</ColumnDef>
  </Columns>

  <Row>
    <Cell>Layout</Cell>
    <Cell>Complete</Cell>
    <Cell>Core</Cell>
  </Row>

  <Row>
    <Cell>Text</Cell>
    <Cell>In Progress</Cell>
    <Cell>Renderer</Cell>
  </Row>
</Table>
```

## 3.5 Tree and TreeItem

```xml
<Tree key="project-tree">
  <TreeItem label="Project" expanded>
    <TreeItem label="src" expanded>
      <TreeItem label="main.rs" />
      <TreeItem label="widgets.rs" />
    </TreeItem>
    <TreeItem label="examples" />
  </TreeItem>
</Tree>
```

## 3.6 Menu and MenuItem

```xml
<Menu key="action-menu">
  <MenuItem key="archive" on-click="archive" shortcut="Ctrl+A">Archive</MenuItem>
  <MenuItem key="duplicate" action="duplicate" shortcut="Ctrl+D">Duplicate</MenuItem>
  <MenuItem key="delete" shortcut="Del" disabled>Delete</MenuItem>
</Menu>
```

## 3.7 ContextMenu

Use `ContextMenu` in a trigger slot.

```xml
<Button key="context-trigger">
  Right-click me
  <ContextMenu slot="context-menu" key="row-context-menu">
    <MenuItem key="copy" action="copy" shortcut="Ctrl+C">Copy</MenuItem>
    <MenuItem key="delete-context" action="delete" shortcut="Del">Delete</MenuItem>
  </ContextMenu>
</Button>
```

---

# 4. Overlay Gallery

## 4.1 Popover

```xml
<Button key="popover-trigger">
  Open Popover
  <Popover slot="popover" key="example-popover">
    <Column padding="8" gap="4">
      <Text heading>Actions</Text>
      <Button key="profile">Profile</Button>
      <Button key="settings">Settings</Button>
    </Column>
  </Popover>
</Button>
```

## 4.2 Tooltip

```xml
<Tooltip key="help-tooltip" text="Helpful text for this area" />
```

## 4.3 Modal

```xml
<Modal key="confirm-modal" open="false" title="Confirmation" close-on-escape close-on-outside-click>
  <Column padding="12" gap="8">
    <Text heading>Confirmation</Text>
    <Text>Modal content goes here.</Text>
    <Row gap="8">
      <Button key="modal-ok" primary on-click="confirm">OK</Button>
      <Button key="modal-cancel" on-click="cancel">Cancel</Button>
    </Row>
  </Column>
</Modal>
```

---

# 5. Primitive Widget Gallery

## 5.1 Icon

```xml
<Row key="icons-demo" gap="8" align-items="center">
  <Icon key="icon-search" name="search" />
  <Icon key="icon-settings" name="settings" />
  <Icon key="icon-home" name="home" />
</Row>
```

## 5.2 Divider

```xml
<Column key="divider-demo" gap="8">
  <Text>Above divider</Text>
  <Divider key="divider" />
  <Text>Below divider</Text>
</Column>
```

---

# 6. Style Gallery

## 6.1 Spacing and Size

```xml
<Column key="spacing-demo" width="320" padding="16" gap="8" background="#ffffff" radius="8">
  <Text>width, padding, and gap</Text>
  <Button width="160">Fixed width</Button>
</Column>
```

## 6.2 Flex Behavior

```xml
<Row key="flex-demo" gap="8" flex-wrap="wrap">
  <Button width="160">One</Button>
  <Button width="160">Two</Button>
  <Button width="160">Three</Button>
</Row>
```

## 6.3 Visual Style

```xml
<Column
  key="card-demo"
  padding="16"
  gap="8"
  background="#f8fafc"
  border="1 #cbd5e1"
  radius="12"
  opacity="0.98"
  shadow="0 4 12 0 rgba(15,23,42,0.25)"
  transform="translate(0,0) scale(1,1) rotate(0)"
>
  <Text heading>Styled Card</Text>
  <Text color="#334155">This demonstrates background, border, radius, opacity, shadow, transform, and text color.</Text>
</Column>
```

## 6.4 Text Style

```xml
<Column key="text-style-demo" gap="4">
  <Text font-size="24" font-weight="bold">Large bold</Text>
  <Text font-style="italic">Italic text</Text>
  <Text color="#dc2626">Red text</Text>
</Column>
```

---

# 7. Full Copy/Paste Gallery

Save this as `rml_widget_gallery.rml`.

```xml
<ScrollArea key="gallery-root" width="100%" height="100%" background="#ffffff">
  <Column key="gallery-page" width="920" padding="16" gap="16">
    <Text key="gallery-title" heading>RML Widget Gallery</Text>
    <Text key="gallery-subtitle">A full showcase of current RML layout, forms, collections, overlays, and primitive widgets.</Text>

    <Column key="layout-section" padding="12" gap="10" background="#f8fafc" radius="8">
      <Text heading>1. Layout Components</Text>

      <Column key="column-demo" padding="12" gap="8" background="#ffffff" radius="8">
        <Text heading>Column</Text>
        <Text>Children are stacked vertically.</Text>
        <Button key="column-button">Button inside column</Button>
      </Column>

      <Row key="row-demo" gap="8" align-items="center" background="#ffffff" padding="12" radius="8">
        <Icon name="search" />
        <TextInput key="row-search" placeholder="Search" />
        <Button key="row-submit" primary>Search</Button>
      </Row>

      <Grid key="grid-demo" grid-template-columns="1fr 1fr" gap="8" background="#ffffff" padding="12" radius="8">
        <Text background="#eef2ff" padding="8">Cell 1</Text>
        <Text background="#ecfeff" padding="8">Cell 2</Text>
        <Text background="#fef9c3" padding="8">Cell 3</Text>
        <Text background="#fce7f3" padding="8">Cell 4</Text>
      </Grid>

      <Stack key="stack-demo" width="240" height="120" background="#ffffff" radius="8">
        <Text padding="12">Stack base layer</Text>
        <Button key="stack-floating" width="120" z-index="2">Floating</Button>
      </Stack>

      <Absolute key="absolute-demo" width="260" height="140" background="#ffffff" radius="8">
        <Text padding="8">Absolute area</Text>
        <Button key="absolute-top-left" position="absolute" inset="40 auto auto 12">Top Left</Button>
        <Button key="absolute-bottom-right" position="absolute" inset="auto 12 12 auto">Bottom Right</Button>
      </Absolute>

      <ScrollArea key="scroll-demo" height="120" axis="y" background="#ffffff" radius="8">
        <Column gap="4" padding="8">
          <Text height="32">Scrollable line 1</Text>
          <Text height="32">Scrollable line 2</Text>
          <Text height="32">Scrollable line 3</Text>
          <Text height="32">Scrollable line 4</Text>
          <Text height="32">Scrollable line 5</Text>
        </Column>
      </ScrollArea>

      <Column key="text-demo" gap="4" background="#ffffff" padding="12" radius="8">
        <Text heading>Text</Text>
        <Text font-size="16" font-weight="normal">Body text</Text>
        <Text color="#2563eb" font-weight="bold">Colored bold text</Text>
      </Column>

      <Canvas key="canvas-demo" name="chart" width="240" height="140" background="#ffffff" />
    </Column>

    <Column key="forms-section" padding="12" gap="10" background="#f8fafc" radius="8">
      <Text heading>2. Form Components</Text>

      <Row key="button-demo" gap="8" align-items="center" flex-wrap="wrap">
        <Button key="button-primary" primary on-click="save">Primary</Button>
        <Button key="button-secondary" variant="secondary">Secondary</Button>
        <Button key="button-disabled" disabled>Disabled</Button>
        <Button key="button-loading" loading>Loading</Button>
      </Row>

      <Column key="input-demo" gap="8">
        <TextInput key="name-input" placeholder="Name" default-value="Alice" aria-label="Name" />
        <Input key="email-input" placeholder="Email" default-value="alice@example.com" />
        <TextInput key="password-input" placeholder="Password" password />
      </Column>

      <Column key="checkbox-demo" gap="8">
        <Checkbox key="enabled-checkbox" checked label="Enabled" />
        <Checkbox key="terms-checkbox">Accept terms</Checkbox>
        <Checkbox key="indeterminate-checkbox" indeterminate label="Partially selected" />
      </Column>

      <Column key="radio-demo" gap="8">
        <Radio key="radio-free" value="free" label="Free" checked />
        <Radio key="radio-pro" value="pro" label="Pro" />
        <Radio key="radio-enterprise" value="enterprise" label="Enterprise" disabled />
      </Column>

      <Row key="select-row" gap="8" flex-wrap="wrap">
        <Select key="priority-select" placeholder="Priority" default-value="medium">
          <SelectStyle part="trigger" height="32" />
          <SelectStyle part="popover" width="220" />
          <SelectStyle part="item" padding="8" />
          <Option value="low">Low</Option>
          <Option value="medium">Medium</Option>
          <Option value="high" disabled>High</Option>
        </Select>

        <Select key="size-select" placeholder="Size">
          <SelectOption value="s">Small</SelectOption>
          <SelectOption value="m">Medium</SelectOption>
          <SelectOption value="l">Large</SelectOption>
        </Select>
      </Row>

      <Textarea key="notes-textarea" placeholder="Notes" default-value="Initial notes" rows="4" />
    </Column>

    <Column key="collections-section" padding="12" gap="10" background="#f8fafc" radius="8">
      <Text heading>3. Collection Components</Text>

      <Tabs key="settings-tabs" active-index="0">
        <Tab label="General" />
        <Tab label="Advanced" />
        <Tab label="About" />
      </Tabs>

      <Tabs key="compact-tabs" tabs="Overview,Details,Activity" active-index="1" />

      <List key="task-list" selected-index="1">
        <ListItem>Inbox</ListItem>
        <ListItem>Today</ListItem>
        <ListItem>Done</ListItem>
      </List>

      <List key="compact-list" items="Inbox,Today,Done" selected-index="0" />

      <Table key="status-table" columns="Name,Status" selected-row="0">
        <TableRow values="Runtime,Ready" />
        <TableRow values="Renderer,Ready" />
        <TableRow values="RML,Ready" />
      </Table>

      <Table key="detailed-table" selected-row="1">
        <Columns>
          <ColumnDef>Name</ColumnDef>
          <ColumnDef>Status</ColumnDef>
          <ColumnDef>Owner</ColumnDef>
        </Columns>
        <Row>
          <Cell>Layout</Cell>
          <Cell>Complete</Cell>
          <Cell>Core</Cell>
        </Row>
        <Row>
          <Cell>Text</Cell>
          <Cell>In Progress</Cell>
          <Cell>Renderer</Cell>
        </Row>
      </Table>

      <Tree key="project-tree">
        <TreeItem label="Project" expanded>
          <TreeItem label="src" expanded>
            <TreeItem label="main.rs" />
            <TreeItem label="widgets.rs" />
          </TreeItem>
          <TreeItem label="examples" />
        </TreeItem>
      </Tree>

      <Menu key="action-menu">
        <MenuItem key="archive" on-click="archive" shortcut="Ctrl+A">Archive</MenuItem>
        <MenuItem key="duplicate" action="duplicate" shortcut="Ctrl+D">Duplicate</MenuItem>
        <MenuItem key="delete" shortcut="Del" disabled>Delete</MenuItem>
      </Menu>

      <Button key="context-trigger">
        Right-click me
        <ContextMenu slot="context-menu" key="row-context-menu">
          <MenuItem key="copy" action="copy" shortcut="Ctrl+C">Copy</MenuItem>
          <MenuItem key="delete-context" action="delete" shortcut="Del">Delete</MenuItem>
        </ContextMenu>
      </Button>
    </Column>

    <Column key="overlays-section" padding="12" gap="10" background="#f8fafc" radius="8">
      <Text heading>4. Overlay Components</Text>

      <Row key="overlay-row" gap="8" flex-wrap="wrap">
        <Button key="popover-trigger">
          Open Popover
          <Popover slot="popover" key="example-popover">
            <Column padding="8" gap="4">
              <Text heading>Actions</Text>
              <Button key="profile">Profile</Button>
              <Button key="settings">Settings</Button>
            </Column>
          </Popover>
        </Button>

        <Tooltip key="help-tooltip" text="Helpful text for this area" />

        <Modal key="confirm-modal" open="false" title="Confirmation" close-on-escape close-on-outside-click>
          <Column padding="12" gap="8">
            <Text heading>Confirmation</Text>
            <Text>Modal content goes here.</Text>
            <Row gap="8">
              <Button key="modal-ok" primary on-click="confirm">OK</Button>
              <Button key="modal-cancel" on-click="cancel">Cancel</Button>
            </Row>
          </Column>
        </Modal>
      </Row>
    </Column>

    <Column key="primitive-section" padding="12" gap="10" background="#f8fafc" radius="8">
      <Text heading>5. Primitive Widgets</Text>

      <Row key="icons-demo" gap="8" align-items="center">
        <Icon key="icon-search" name="search" />
        <Icon key="icon-settings" name="settings" />
        <Icon key="icon-home" name="home" />
      </Row>

      <Column key="divider-demo" gap="8">
        <Text>Above divider</Text>
        <Divider key="divider" />
        <Text>Below divider</Text>
      </Column>
    </Column>

    <Column key="style-section" padding="12" gap="10" background="#f8fafc" radius="8">
      <Text heading>6. Style Examples</Text>

      <Column key="spacing-demo" width="320" padding="16" gap="8" background="#ffffff" radius="8">
        <Text>width, padding, and gap</Text>
        <Button width="160">Fixed width</Button>
      </Column>

      <Row key="flex-demo" gap="8" flex-wrap="wrap">
        <Button width="160">One</Button>
        <Button width="160">Two</Button>
        <Button width="160">Three</Button>
      </Row>

      <Column key="card-demo" padding="16" gap="8" background="#f8fafc" border="1 #cbd5e1" radius="12" opacity="0.98" shadow="0 4 12 0 rgba(15,23,42,0.25)" transform="translate(0,0) scale(1,1) rotate(0)">
        <Text heading>Styled Card</Text>
        <Text color="#334155">This demonstrates background, border, radius, opacity, shadow, transform, and text color.</Text>
      </Column>

      <Column key="text-style-demo" gap="4">
        <Text font-size="24" font-weight="bold">Large bold</Text>
        <Text font-style="italic">Italic text</Text>
        <Text color="#dc2626">Red text</Text>
      </Column>
    </Column>
  </Column>
</ScrollArea>
```

---

# 8. Acceptance Checklist

## Parser coverage

- [ ] Every supported tag parses.
- [ ] `Canvas` errors without `name`.
- [ ] `Icon` errors without `name`.
- [ ] Unknown tags error.
- [ ] Invalid booleans error.
- [ ] Invalid enum values error.
- [ ] Unsupported `style` warns.
- [ ] Text child normalization works.
- [ ] `<Text>` rejects child elements.

## Runtime coverage

- [ ] Every gallery section creates layout boxes.
- [ ] Interactive widgets create hit-test entries.
- [ ] Text renders visibly.
- [ ] ScrollArea clips content.
- [ ] Popover opens from trigger.
- [ ] ContextMenu opens from secondary click.
- [ ] Modal renders when opened.
- [ ] Select options render and can be selected.
- [ ] List selection updates state.
- [ ] Table row selection updates state.
- [ ] Tabs active index updates state.

## Documentation coverage

- [ ] Every supported widget has a basic example.
- [ ] Every layout primitive has an example.
- [ ] Collections have compact and expanded examples where supported.
- [ ] Current incomplete/placeholder behaviors are clearly marked.
