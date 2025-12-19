package org.ori;

import android.content.Context;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputMethodManager;
import android.text.InputType;
import android.view.KeyEvent;

public final class RustView extends View {
    final InputMethodManager inputMethodManager;
    long context;

    public RustView(Context context) {
        super(context);
        setFocusable(true);
        setFocusableInTouchMode(true);
        requestFocus();

        inputMethodManager = (InputMethodManager) context.getSystemService(Context.INPUT_METHOD_SERVICE);
    }

    @Override
    public InputConnection onCreateInputConnection(EditorInfo outAttrs) {
        outAttrs.inputType = InputType.TYPE_CLASS_TEXT;

        return new RustInputConnection(this);
    }

    public native String getTextBeforeCursorNative(int n, int flags);

    public native String getTextAfterCursorNative(int n, int flags);

    public native String getSelectedTextNative(int flags);

    public native boolean commitTextNative(String text, int newCursorPosition);

    public native boolean deleteSurroundingTextNative(int beforeLength, int afterLength);

    public native boolean deleteSurroundingTextInCodePointsNative(int beforeLength, int afterLength);

    public native boolean setComposingTextNative(String text, int newCursorPosition);

    public native boolean sendKeyEventNative(KeyEvent event);

    public native boolean setSelectionNative(int start, int end);
}
