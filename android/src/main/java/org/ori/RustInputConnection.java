package org.ori;

import android.view.View;
import android.view.inputmethod.BaseInputConnection;
import android.view.inputmethod.InputConnection;
import android.view.KeyEvent;

public final class RustInputConnection extends BaseInputConnection {
    final RustView view;

    public RustInputConnection(RustView view) {
        super(view, true);
        this.view = view;
    }

    @Override
    public CharSequence getTextBeforeCursor(int n, int flags) {
        return view.getTextBeforeCursorNative(n, flags);
    }

    @Override
    public CharSequence getTextAfterCursor(int n, int flags) {
        return view.getTextAfterCursorNative(n, flags);
    }

    @Override
    public CharSequence getSelectedText(int flags) {
        return view.getSelectedTextNative(flags);
    }

    @Override
    public boolean commitText(CharSequence text, int newCursorPosition) {
        return view.commitTextNative(text.toString(), newCursorPosition);
    }

    @Override
    public boolean deleteSurroundingText(int beforeLength, int afterLength) {
        return view.deleteSurroundingTextNative(beforeLength, afterLength);
    }

    @Override
    public boolean deleteSurroundingTextInCodePoints(int beforeLength, int afterLength) {
        return view.deleteSurroundingTextInCodePointsNative(beforeLength, afterLength);
    }

    @Override
    public boolean setComposingText(CharSequence text, int newCursorPosition) {
        return view.setComposingTextNative(text.toString(), newCursorPosition);
    }

    @Override
    public boolean sendKeyEvent(KeyEvent event) {
        return view.sendKeyEventNative(event);
    }

    @Override
    public boolean setSelection(int start, int end) {
        return view.setSelectionNative(start, end);
    }
}
