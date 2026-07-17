export type TComboboxSection<T> = {
    key: string;
    heading: string;
    items: T[];
    /** Non-selectable sections render informational rows (e.g. "already on the map"). */
    selectable?: boolean;
};

export type TComboboxRow<T> = { kind: 'heading'; label: string } | { kind: 'option'; value: T; section: string; selectable: boolean };

/**
 * reka's ComboboxVirtualizer only handles a flat list, so grouped options are
 * flattened into a single row list with headings as non-interactive rows of
 * their own. Empty sections disappear entirely, heading included.
 */
export function flattenComboboxSections<T>(sections: TComboboxSection<T>[]): TComboboxRow<T>[] {
    return sections.flatMap((section): TComboboxRow<T>[] =>
        section.items.length === 0
            ? []
            : [
                  ...(section.heading === '' ? [] : [{ kind: 'heading', label: section.heading } satisfies TComboboxRow<T>]),
                  ...section.items.map(
                      (value): TComboboxRow<T> => ({
                          kind: 'option',
                          value,
                          section: section.key,
                          selectable: section.selectable ?? true,
                      }),
                  ),
              ],
    );
}

/** Typeahead text for a flattened row: option label or nothing for headings. */
export function comboboxRowText<T>(row: TComboboxRow<T>, label: (value: T) => string): string {
    return row.kind === 'option' ? label(row.value) : '';
}
