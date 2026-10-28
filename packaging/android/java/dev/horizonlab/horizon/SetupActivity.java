package dev.horizonlab.horizon;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.DialogInterface;
import android.content.Intent;
import android.content.SharedPreferences;
import android.graphics.Color;
import android.net.ConnectivityManager;
import android.os.Bundle;
import android.os.Handler;
import android.os.StatFs;
import android.view.Gravity;
import android.view.View;
import android.view.WindowManager;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ProgressBar;
import android.widget.ScrollView;
import android.widget.TextView;
import java.io.File;

/**
 * The launcher activity. Starts the game when the resources are ready; otherwise shows the
 * first-run screen that downloads and prepares the game files on the phone (android_setup.rs).
 */
public class SetupActivity extends Activity
        implements View.OnClickListener, DialogInterface.OnClickListener, Runnable {
    static {
        System.loadLibrary("horizon");
    }

    static native int nativeReady(String home);
    static native String nativeDefaultUrl();
    static native void nativeStart(String home, String derivedUrl);
    static native String nativeStatus();

    private static final long NEED_BYTES = 1500L * 1024 * 1024;

    private String home;
    private TextView status;
    private ProgressBar bar;
    private Button button;
    private EditText url;
    private final Handler handler = new Handler();

    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);
        File dir = getExternalFilesDir(null);
        if (dir == null) dir = getFilesDir();
        dir.mkdirs();
        home = dir.getAbsolutePath();
        if (nativeReady(home) == 1) {
            launch();
            return;
        }
        int pad = dp(16);
        LinearLayout box = new LinearLayout(this);
        box.setOrientation(LinearLayout.VERTICAL);
        box.setPadding(pad * 2, pad, pad * 2, pad);
        box.setBackgroundColor(Color.rgb(24, 24, 28));
        TextView title = text("Horizon", 26);
        box.addView(title);
        box.addView(text("Нужно скачать файлы игры Minecraft 26.3 (около 700 МБ) с серверов Mojang. "
                + "Лучше по Wi-Fi.\nThe Minecraft 26.3 game files (about 700 MB) need to be downloaded "
                + "from Mojang. Wi-Fi recommended.", 15));
        box.addView(text("Derived pack URL (Horizon exporter data):", 13));
        url = new EditText(this);
        url.setSingleLine(true);
        url.setTextColor(Color.WHITE);
        url.setTextSize(13);
        url.setBackgroundColor(Color.rgb(52, 52, 60));
        url.setHintTextColor(Color.GRAY);
        url.setPadding(dp(8), dp(8), dp(8), dp(8));
        SharedPreferences prefs = getSharedPreferences("setup", MODE_PRIVATE);
        url.setText(prefs.getString("derived_url", nativeDefaultUrl()));
        if (new File(dir, "horizon-derived.zip").isFile()) {
            url.setHint("horizon-derived.zip found");
        }
        box.addView(url);
        button = new Button(this);
        button.setText("Скачать файлы игры / Download game files");
        button.setOnClickListener(this);
        box.addView(button);
        bar = new ProgressBar(this, null, android.R.attr.progressBarStyleHorizontal);
        bar.setMax(1000);
        box.addView(bar);
        status = text("", 14);
        box.addView(status);
        ScrollView scroll = new ScrollView(this);
        scroll.setBackgroundColor(Color.rgb(24, 24, 28));
        scroll.addView(box);
        setContentView(scroll);
        // A run still going (the activity was recreated) keeps showing its progress.
        if (android.os.Build.VERSION.SDK_INT >= 33) {
            requestPermissions(new String[] {"android.permission.POST_NOTIFICATIONS"}, 1);
        }
        if (nativeStatus().contains("\trun\t")) {
            begin();
        }
    }

    private int dp(int v) {
        return (int) (v * getResources().getDisplayMetrics().density);
    }

    private TextView text(String s, int size) {
        TextView t = new TextView(this);
        t.setText(s);
        t.setTextSize(size);
        t.setTextColor(Color.WHITE);
        t.setPadding(0, dp(6), 0, dp(6));
        return t;
    }

    private void confirm() {
        StatFs fs = new StatFs(home);
        if (fs.getAvailableBytes() < NEED_BYTES) {
            status.setText("Мало места: нужно ~1.5 ГБ свободно / Not enough space: ~1.5 GB free needed ("
                    + (fs.getAvailableBytes() >> 20) + " MB free)");
            return;
        }
        ConnectivityManager cm = (ConnectivityManager) getSystemService(Context.CONNECTIVITY_SERVICE);
        if (cm != null && cm.isActiveNetworkMetered()) {
            new AlertDialog.Builder(this)
                    .setTitle("Мобильная сеть / Mobile data")
                    .setMessage("Вы не на Wi-Fi. Скачать ~700 МБ? / You are not on Wi-Fi. Download ~700 MB?")
                    .setPositiveButton("OK", this)
                    .setNegativeButton("Cancel", null)
                    .show();
        } else {
            begin();
        }
    }

    private void begin() {
        String u = url.getText().toString().trim();
        getSharedPreferences("setup", MODE_PRIVATE).edit().putString("derived_url", u).commit();
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        button.setEnabled(false);
        Intent service = new Intent(this, SetupService.class);
        service.putExtra("home", home);
        service.putExtra("url", u);
        if (android.os.Build.VERSION.SDK_INT >= 26) {
            startForegroundService(service);
        } else {
            startService(service);
        }
        handler.post(poll);
    }

    static String stageName(String s) {
        if (s.equals("version")) return "Версия / Version";
        if (s.equals("client")) return "Клиент / Client JAR";
        if (s.equals("index")) return "Индекс / Asset index";
        if (s.equals("extract")) return "Распаковка / Extracting";
        if (s.equals("assets")) return "Звуки, шрифты, языки / Sounds, fonts, languages";
        if (s.equals("derived")) return "Данные Horizon / Derived pack";
        return s;
    }

    public void onClick(View v) {
        confirm();
    }

    public void onClick(DialogInterface d, int w) {
        begin();
    }

    private final Runnable poll = this;

    public void run() {
        {
            String[] p = nativeStatus().split("\t", -1);
            if (p.length < 7) {
                handler.postDelayed(this, 300);
                return;
            }
            long files = Long.parseLong(p[1]), filesTotal = Long.parseLong(p[2]);
            long bytes = Long.parseLong(p[3]), bytesTotal = Long.parseLong(p[4]);
            String line = stageName(p[0]);
            if (filesTotal > 0) line += "\n" + files + " / " + filesTotal + " files";
            if (bytesTotal > 0) {
                line += "   " + (bytes >> 20) + " / " + (bytesTotal >> 20) + " MB";
                bar.setProgress((int) (1000 * bytes / Math.max(1, bytesTotal)));
            } else if (filesTotal > 0) {
                bar.setProgress((int) (1000 * files / Math.max(1, filesTotal)));
            }
            if (p[5].equals("done")) {
                getWindow().clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
                launch();
                return;
            }
            if (p[5].equals("error")) {
                getWindow().clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
                status.setText("Ошибка / Error: " + p[6] + "\n\nНажмите ещё раз, загрузка продолжится. "
                        + "/ Tap again, it resumes.");
                button.setText("Повторить / Retry");
                button.setEnabled(true);
                return;
            }
            status.setText(line);
            handler.postDelayed(this, 300);
        }
    }

    private void launch() {
        startActivity(new Intent(this, HorizonActivity.class));
        finish();
    }

    @Override
    protected void onDestroy() {
        handler.removeCallbacks(poll);
        super.onDestroy();
    }
}
