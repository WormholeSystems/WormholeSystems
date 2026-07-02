/**
 * Casts a boolean to a empty data-attribute value
 *
 * @example
 * <Component :data-attribute="Data(true)" /> // renders as <Component data-attribute />
 * @param value
 * @constructor
 */
export function Data(value: boolean | null | undefined) {
    return value ? '' : null;
}
