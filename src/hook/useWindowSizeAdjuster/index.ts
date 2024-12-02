import { invoke } from '@tauri-apps/api/core';
import { RefObject, useCallback, useEffect, useState } from 'react';

export const useWindowSizeAdjuster = (
    contentRef: RefObject<HTMLDivElement>,
    dependencies: any[] = []
) => {
    const storageAutoAdjustSize = localStorage.getItem('autoAdjustSize');

    const [autoAdjustSize, setAutoAdjustSize] = useState(!storageAutoAdjustSize || storageAutoAdjustSize === 'true');

    const toggleAutoAdjustSize = () => {
        setAutoAdjustSize((prev) => {
            localStorage.setItem('autoAdjustSize', (!prev).toString());
            return !prev;
        });
    };


    const adjustWindowSize = useCallback(async () => {
        if (contentRef.current) {
            const { scrollWidth, scrollHeight } = contentRef.current;


            try {
                await invoke('set_window_size', {
                    width: scrollWidth,
                    height: scrollHeight,
                });
            } catch (error) {
                console.error('Error adjusting window size:', error);
            }
        }
    }, [contentRef]);

    useEffect(() => {
        if (autoAdjustSize) {
            adjustWindowSize();
        }
    }, [autoAdjustSize, adjustWindowSize, ...dependencies]);


    return { adjustWindowSize, toggleAutoAdjustSize, autoAdjustSize };
};