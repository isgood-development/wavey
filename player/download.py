import threading
import pafy
import os
import shutil
from youtubesearchpython import VideosSearch
from moviepy.video.io.VideoFileClip import VideoFileClip # Importing from submodule as moviepy.editor is intended for manual use.
from tkinter import *
from tkinter import messagebox
from tkinter import filedialog
from tkinter.font import Font

import player.data as data

def file_convert(title):
    mp4 = f'./data/audio/{title}.mp4'
    mp3 = f'./data/audio/{title}.mp3'
    clip = VideoFileClip(mp4)
    audioclip = clip.audio
    audioclip.write_audiofile(mp3)
    audioclip.close()
    clip.close()
    os.remove(f'./data/audio/{title}.mp4')

def download(*, title=None, link=None):
    win.destroy()
    global root
    root = Toplevel()
    root.wm_title("Progress")
    root.wm_attributes("-topmost", 1)
    root.configure(bg=data.view("back_colour", "c"))
    global prog_label
    prog_label = Label(
        root,
        text="Starting download...",
        font=Font(size=14, family="Cascadia Mono"),
        fg="white",
        bg=data.view("back_colour", "c")
    )
    prog_label.pack()

    def _callback(total, recvd, ratio, rate, eta):
        prog_label.configure(text=f"Downloading: {round(ratio*100)}% [{eta}s]")

    def clean_name(title):
        """Removes any illegal file characters from the name given from YT"""
        bannedchars = ["<", ">", ":", "\"", "/", "\\", "|", "?", "*"] # banned characters in Windows.
        for char in bannedchars:                                      # may not apply to other operating systems.
            if char in title:
                title = title.replace(char, "_")
        
        return title
    
    if not link and title:
        search = VideosSearch(title, limit=1)
        search = search.result()

        result = search["result"][0]
        url = result["link"]
        title = result["title"]
        
        video = pafy.new(url)   
    
    else:
        try:
            video = pafy.new(link)
        except ValueError:
            return messagebox.showerror(
                title="Invalid URL",
                message="The URL provided is invalid. Please provide the 11 character video id or the URL to the video."
            )
        title = video.title
        link = video.watchv_url
    
    print(f"Downloading: {title} ({link})")
    video = video.getbest()
    video.download(
        filepath="./data/audio/",
        callback=_callback
    )
    prog_label.configure(text="Converting to audio")
    title = clean_name(title)
    file_convert(title)
    root.destroy()
    messagebox.showinfo(
        title="Song Downloaded",
        message=f"Downloaded:\n{title}"
    )

def file_opener():
    file = filedialog.askopenfile(
        initialdir=".",
        title="Select an MP3 file",
        filetypes=(
            ("MP3 files", "*.mp3"),
            ("all files", "*.*")
        )
    )
    if not file:
        return

    filename = file.name.split("/")
    filename = filename[len(filename)-1]
    shutil.copyfile(file.name, f"./data/audio/{filename}")
    # TODO: update song list

def download_window():
    back_colour = data.view("back_colour", "c")
    fore_colour = data.view("fore_colour", "c")

    window = Toplevel()
    window.configure(bg=data.view("back_colour", "c"))
    window.geometry("300x300")
    window.wm_title("Add music")
    window.resizable(False, False)
    
    try:
        window.iconbitmap("player/Assets/downloadicon.ico")
    except TclError:
        pass

    global win
    win = window
    
    Label(
        window,
        text="Add song by name:",
        bg=back_colour,
        fg="white",
        font=Font(size=10, family="Cascadia Mono")
    ).pack()

    add_name_entry = Entry(
        window,
        fg="white",
        bg=fore_colour,
        font=Font(size=10, family="Cascadia Mono")
    )
    add_name_entry.pack()
    
    download_name = Button(
        window,
        fg="white",
        bg=fore_colour,
        font=Font(size=10, family="Cascadia Mono"),
        text="Download",
        borderwidth=0,
        command=lambda: threading.Thread(
            target=lambda: download(
                title=add_name_entry.get()
            )
        ).start()
    )
    download_name.pack()

    Label(
        window,
        text="Add song by YT URL:",
        fg="white",
        bg=back_colour,
        font=Font(size=10, family="Cascadia Mono")
    ).pack()

    add_by_url_entry = Entry(
        window,
        fg="white",
        bg=fore_colour,
        font=Font(size=10, family="Cascadia Mono")
    )
    add_by_url_entry.pack()

    download_url = Button(
        window,
        fg="white",
        bg=fore_colour,
        font=Font(size=10, family="Cascadia Mono"),
        text="Download",
        borderwidth=0,
        command=lambda: threading.Thread(
            target=lambda: download(
                link=add_by_url_entry.get()
            )
        ).start()
    )
    download_url.pack()

    import_music = Button(
        window,
        fg="white",
        bg=fore_colour,
        font=Font(size=10, family="Cascadia Mono"),
        text="Import Music from PC",
        command=file_opener
    )
    import_music.pack(pady=20)