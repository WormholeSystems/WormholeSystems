# Resizable & Moveable Layout System

## Overview

The map view now features a fully customizable, resizable, and moveable layout system that allows users to personalize their workspace across three breakpoints (Small, Medium, Large).

## Features

- **Drag & Drop**: Move sections around by dragging them in edit mode
- **Resizable**: Resize sections by dragging the resize handles
- **Responsive**: Three breakpoints (sm/md/lg) with independent layouts
- **Persistent**: Layout configurations are saved per user per map
- **Visual Feedback**: Clear visual cues when in edit mode

## Breakpoints

### Small (sm) - Mobile & Tablet
- **Min Width**: 640px
- **Grid Columns**: 1
- **Use Case**: Mobile devices and small tablets

### Medium (md) - Tablet & Small Desktop
- **Min Width**: 1024px
- **Grid Columns**: 2
- **Use Case**: Tablets in landscape and small desktop screens

### Large (lg) - Desktop & Wide Screen
- **Min Width**: 1536px
- **Grid Columns**: 10
- **Use Case**: Desktop and wide screen monitors

## Usage

### Entering Edit Mode

1. Click the **Edit Layout** button (pencil icon) in the top-right corner
2. The layout will become editable, and the breakpoint selector will appear

### Selecting a Breakpoint

When in edit mode, you can select which breakpoint layout to edit:
- Click on the **Smartphone** icon for small breakpoint
- Click on the **Tablet** icon for medium breakpoint
- Click on the **Monitor** icon for large breakpoint

### Moving Sections

1. Enter edit mode
2. Click and hold on any section
3. Drag it to the desired position
4. Release to drop it in place

### Resizing Sections

1. Enter edit mode
2. Hover over a section to reveal the resize handle (bottom-right corner)
3. Click and drag the handle to resize
4. Each section has minimum height constraints to ensure content remains visible

### Saving Changes

1. After making your changes, click the **Save** button (floppy disk icon)
2. Your layout configuration will be saved to the database
3. The layout will persist across sessions

### Resetting to Default

If you want to revert to the default layout for the current breakpoint:
1. Enter edit mode
2. Select the breakpoint you want to reset
3. Click the **Reset** button (circular arrow icon)
4. Save your changes

## Technical Details

### Files Structure

```
resources/js/
├── types/
│   └── layout.ts                    # Type definitions
├── composables/
│   └── useMapLayout.ts              # Layout state management
├── components/
│   └── layout/
│       └── LayoutEditor.vue         # Layout editor controls
└── pages/
    └── maps/
        └── ShowMap.vue              # Main map view with grid layout
```

### Database Schema

Layout configurations are stored in the `map_user_settings` table:

- `layout_config_sm`: JSON field storing small breakpoint layout
- `layout_config_md`: JSON field storing medium breakpoint layout
- `layout_config_lg`: JSON field storing large breakpoint layout

Each layout configuration contains:
```typescript
{
  cols: number;           // Number of grid columns
  rowHeight: number;      // Height of each row in pixels
  items: [
    {
      i: string;          // Component identifier
      x: number;          // X position in grid units
      y: number;          // Y position in grid units
      w: number;          // Width in grid units
      h: number;          // Height in grid units
      minH?: number;      // Minimum height
      static?: boolean;   // Whether draggable/resizable
    }
  ]
}
```

### Available Sections

The following sections can be arranged:

1. **map** - Main map visualization
2. **solarsystem** - Solar system details
3. **signatures** - Signature list (conditional)
4. **audits** - Audit history (conditional)
5. **ship-history** - Ship history tracking
6. **characters** - Map characters list
7. **killmails** - Recent killmails
8. **autopilot** - Autopilot routing

Note: Some sections are conditional and only appear when relevant data is available.

### Grid Layout Library

The system uses [grid-layout-plus](https://github.com/MagicCube/grid-layout-plus), a Vue 3 compatible drag-and-drop grid layout library.

## API Endpoints

Layout configurations are saved via the existing `MapUserSettingController`:

```
PUT /api/map-user-settings/{id}
```

Request body:
```json
{
  "layout_config_sm": { /* layout config */ },
  "layout_config_md": { /* layout config */ },
  "layout_config_lg": { /* layout config */ }
}
```

## Customization

### Adding New Sections

To add a new section to the layout:

1. Add the section identifier to `DEFAULT_LAYOUT_CONFIGS` in `resources/js/types/layout.ts`
2. Add a new `<GridItem>` in `ShowMap.vue` with matching identifier
3. Update the default positions for all breakpoints

### Changing Default Layouts

Default layouts can be modified in `resources/js/types/layout.ts`:

```typescript
export const DEFAULT_LAYOUT_CONFIGS: LayoutConfigs = {
  sm: { /* small breakpoint config */ },
  md: { /* medium breakpoint config */ },
  lg: { /* large breakpoint config */ }
};
```

## Browser Support

The layout system is compatible with all modern browsers that support:
- CSS Transforms
- Touch Events (for mobile)
- ES6+ JavaScript

## Performance Considerations

- Layout updates use CSS transforms for optimal performance
- Grid calculations are debounced to reduce unnecessary reflows
- Section content is not re-rendered during drag/resize operations
- Layout configurations are cached in memory during editing

## Future Enhancements

Potential improvements for future versions:

- [ ] Layout templates/presets
- [ ] Share layouts between users
- [ ] Import/export layout configurations
- [ ] More granular grid controls
- [ ] Section visibility toggles
- [ ] Mobile-specific optimizations
- [ ] Undo/redo functionality

