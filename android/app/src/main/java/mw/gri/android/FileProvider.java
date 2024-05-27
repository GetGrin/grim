package mw.gri.android;

public class FileProvider extends androidx.core.content.FileProvider {
    public FileProvider() {
        super(R.xml.paths);
    }
}
