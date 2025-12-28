/**
 * SCROLL ANCHOR COMPONENT - Cognitive Latency V2.1
 * 
 * Purpose: Preserve user's spatial reference during data updates.
 * Philosophy: "Anchor is the cognitive coordinate system."
 * 
 * Without anchor, ghosting loses meaning - users don't know if they're
 * looking at the same data or different data.
 */

import React, { useEffect, useRef, useState } from 'react';

export interface ScrollAnchorProps {
    /**
     * Unique identifier for the current row/item being anchored
     */
    dataKey: string;

    /**
     * Callback when the anchored row disappears from dataset
     */
    onAnchorLost?: () => void;

    /**
     * Children to render (typically a table or list)
     */
    children: React.ReactNode;

    /**
     * Optional: Show visual indicator of anchor
     */
    showIndicator?: boolean;
}

/**
 * ScrollAnchor Component
 * 
 * Maintains scroll position relative to a specific data row.
 * When data updates, re-scrolls to keep that row in view.
 * 
 * Usage:
 * <ScrollAnchor dataKey={currentRowId}>
 *   <RiskTable data={risks} />
 * </ScrollAnchor>
 */
export const ScrollAnchor: React.FC<ScrollAnchorProps> = ({
    dataKey,
    onAnchorLost,
    children,
    showIndicator = false,
}) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const [anchoredElement, setAnchoredElement] = useState<HTMLElement | null>(null);
    const previousDataKey = useRef<string>(dataKey);

    useEffect(() => {
        if (!containerRef.current) return;

        // Find the element with the current dataKey
        const findAnchoredElement = (): HTMLElement | null => {
            const elements = containerRef.current?.querySelectorAll(`[data-key="${dataKey}"]`);
            return elements && elements.length > 0 ? (elements[0] as HTMLElement) : null;
        };

        const element = findAnchoredElement();

        if (element) {
            setAnchoredElement(element);

            // If dataKey changed, scroll to new anchor
            if (previousDataKey.current !== dataKey) {
                element.scrollIntoView({ block: 'nearest', behavior: 'auto' });
                previousDataKey.current = dataKey;
            }
        } else if (previousDataKey.current !== dataKey) {
            // Anchor lost
            setAnchoredElement(null);
            onAnchorLost?.();
            previousDataKey.current = dataKey;
        }
    }, [dataKey, onAnchorLost]);

    // Apply visual indicator if enabled
    useEffect(() => {
        if (!showIndicator || !anchoredElement) return;

        anchoredElement.style.borderLeft = '3px solid var(--info-color)';
        anchoredElement.style.transition = 'border-left 0.2s ease';

        return () => {
            anchoredElement.style.borderLeft = '';
            anchoredElement.style.transition = '';
        };
    }, [anchoredElement, showIndicator]);

    return (
        <div ref={containerRef} style={{ position: 'relative', height: '100%', overflow: 'auto' }}>
            {children}
        </div>
    );
};

/**
 * Hook for managing scroll anchor state
 * 
 * Usage:
 * const { anchorKey, setAnchorKey } = useScrollAnchor();
 * 
 * // When user clicks a row:
 * setAnchorKey(row.id);
 */
export function useScrollAnchor(initialKey?: string) {
    const [anchorKey, setAnchorKey] = useState<string>(initialKey || '');

    return {
        anchorKey,
        setAnchorKey,
    };
}

/**
 * Utility: Add data-key attribute to table rows
 * 
 * Usage in RiskTable:
 * <tr {...getAnchorProps(risk.id)}>
 */
export function getAnchorProps(key: string) {
    return {
        'data-key': key,
    };
}
