require("wx")


function createControls(parent)
    local scrollWin = wx.wxScrolledWindow(parent, ID_PARENT_SCROLLEDWINDOW,
                                    wx.wxDefaultPosition, wx.wxDefaultSize,
                                    wx.wxHSCROLL + wx.wxVSCROLL)
    scrollWin:SetScrollbars(15, 15, 400, 1000, 0, 0, false)

    
end

function main()
    frame = wx.wxFrame(wx.NULL, wx.wxID_ANY, 
                                    "Loadable Maker",
                                    wx.wxPoint(350, 10), -- let system place the frame
                                    wx.wxSize(600, 700),  -- set the size of the frame
                                    wx.wxDEFAULT_FRAME_STYLE )

    --panel = wx.wxPanel(frame, wx.wxID_ANY)
    -- -----------------------------------------------------------------------
    -- Create the menu bar
    local fileMenu = wx.wxMenu()
    fileMenu:Append(wx.wxID_EXIT, "&Exit", "Quit the program")

    local menuBar = wx.wxMenuBar()
    menuBar:Append(fileMenu, "&File")

    frame:SetMenuBar(menuBar)

    frame:CreateStatusBar(1)
    frame:SetStatusText("Welcome to wxLua.")

    frame:Connect(wx.wxID_EXIT, wx.wxEVT_COMMAND_MENU_SELECTED,
        function (event)
            frame:Close(true)
        end )

    frame:Show(true)
end

main()

wx.wxGetApp():MainLoop()