package org.ori;

import android.app.NativeActivity;
import android.os.Bundle;

public final class RustActivity extends NativeActivity {
    RustView rustView;

    @Override
    protected void onCreate(Bundle savedInstanceData) {
        super.onCreate(savedInstanceData);

        rustView = new RustView(this);
        setContentView(rustView);
    }
}
