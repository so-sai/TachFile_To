/**
 * GHOSTING TRANSITION ENGINE - Cognitive Latency V2.1
 * 
 * Purpose: Rule-based 2-frame transition for context-preserving data updates.
 * Philosophy: Ghosting is a "cognitive bridge", not a decoration.
 * 
 * Guardrail #1: All transitions MUST pass rule validation.
 * Dev violations trigger console.warn("[CognitiveViolation]").
 */

export type GhostingContext =
    | 'fileUpload'
    | 'dataRefresh'
    | 'contextSwitch'
    | 'sort'
    | 'pagination'
    | 'inlineEdit'
    | 'selectionChange';

export interface GhostingConfig {
    enabled: boolean;
    duration: number; // milliseconds (default: 33ms = 2 frames at 60fps)
    effect: 'opacity' | 'none';
}

/**
 * Rule Set: Defines when ghosting is allowed/forbidden
 */
const GHOSTING_RULES = {
    allowed: new Set<GhostingContext>([
        'fileUpload',      // Major data change (Δrows > 50%)
        'dataRefresh',     // Background update
        'contextSwitch',   // File/project switch
    ]),

    forbidden: new Set<GhostingContext>([
        'sort',            // User expects instant reorder
        'pagination',      // Spatial continuity expected
        'inlineEdit',      // Direct manipulation
        'selectionChange', // Immediate feedback required
    ]),
};

/**
 * Feature flag: Can be toggled via environment or runtime config
 */
let ENABLE_GHOSTING = true;

export function setGhostingEnabled(enabled: boolean): void {
    ENABLE_GHOSTING = enabled;
}

export function isGhostingEnabled(): boolean {
    return ENABLE_GHOSTING;
}

/**
 * Validates if ghosting is allowed for a given context
 * 
 * @param context - The type of data update
 * @param deltaRows - Optional: percentage of rows changed (0-100)
 * @param datasetSize - Optional: total number of rows
 * @returns true if ghosting is allowed
 */
export function isGhostingAllowed(
    context: GhostingContext,
    deltaRows?: number,
    datasetSize?: number
): boolean {
    // Feature flag check
    if (!ENABLE_GHOSTING) {
        return false;
    }

    // Forbidden contexts always return false
    if (GHOSTING_RULES.forbidden.has(context)) {
        if (process.env.NODE_ENV === 'development') {
            console.warn(
                `[CognitiveViolation] Ghosting attempted in forbidden context: "${context}"`
            );
        }
        return false;
    }

    // Allowed contexts
    if (GHOSTING_RULES.allowed.has(context)) {
        // Additional heuristics for allowed contexts
        if (deltaRows !== undefined && deltaRows < 50) {
            // Small changes don't need ghosting
            return false;
        }

        if (datasetSize !== undefined && datasetSize < 10000) {
            // Small datasets render fast enough without ghosting
            return false;
        }

        return true;
    }

    // Unknown context: deny by default
    if (process.env.NODE_ENV === 'development') {
        console.warn(
            `[CognitiveViolation] Unknown ghosting context: "${context}". Denying by default.`
        );
    }
    return false;
}

/**
 * Apply ghosting transition to an element
 * 
 * @param element - DOM element to apply ghosting to
 * @param context - The update context
 * @param config - Optional configuration override
 */
export function applyGhosting(
    element: HTMLElement,
    context: GhostingContext,
    config?: Partial<GhostingConfig>
): void {
    const defaultConfig: GhostingConfig = {
        enabled: true,
        duration: 33, // 2 frames at 60fps
        effect: 'opacity',
    };

    const finalConfig = { ...defaultConfig, ...config };

    if (!finalConfig.enabled || !isGhostingAllowed(context)) {
        return;
    }

    // Apply 2-frame opacity transition
    if (finalConfig.effect === 'opacity') {
        element.style.transition = `opacity ${finalConfig.duration}ms linear`;
        element.style.opacity = '0.5';

        // Force reflow
        void element.offsetHeight;

        // Return to full opacity
        element.style.opacity = '1';

        // Cleanup after transition
        setTimeout(() => {
            element.style.transition = '';
        }, finalConfig.duration);
    }
}

/**
 * React hook for ghosting transitions
 * 
 * Usage:
 * const ghostRef = useGhosting('fileUpload');
 * <div ref={ghostRef}>...</div>
 */
export function useGhosting(context: GhostingContext) {
    return (element: HTMLElement | null) => {
        if (element && isGhostingAllowed(context)) {
            applyGhosting(element, context);
        }
    };
}
