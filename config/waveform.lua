-- 波形解析配置脚本
--协议：AA + 数据长度(通道号 + 数据长度) + 通道号（0-9，大于9不绘制曲线）+ 4字节数据 + 4字节数据 + ...
--如（DATA_TYPE = "int" 通道1，数值：305419896, -305419896） : AA 09 01 12 34 56 78 ED CB A9 88  // 长度9 = 1(通道号) + 8(两组数据)
--如（DATA_TYPE = "float" 通道1，数值：10.0, -10.0） : AA 09 03 00 00 20 41 00 00 20 C1  // 长度9 = 1(通道号) + 8(两组数据)
-- 用户配置区
FRAME_LENGTH = 11  -- 帧长度
BYTES_PER_POINT = 4  -- 每个数据点的字节数
DATA_TYPE = "int"  -- 可选: "int" 或 "float"

function parse_waveform(data)
    -- 检查数据头和最小长度
    if #data < 3 then
        print("数据长度不足: ", #data)
        return nil
    end
    
    if data[1] ~= 0xAA then
        print("数据头错误: ", string.format("%02X", data[1]))
        return nil
    end
    
    local length = data[2]  -- 获取数据长度
    local channel = data[3] -- 获取通道号
    print("数据包长度: ", length, "通道号: ", channel)
    
    -- 修改长度检查，length现在包含通道号的1字节
    if #data < length + 1 then  -- +1是因为包含了AA字节
        print("数据包不完整: 需要", length + 1, "字节, 实际", #data, "字节")
        return nil
    end
    
    -- 解析所有Y数据
    local points = {}
    local index = 4  -- 从第4个字节开始是Y数据
    while index <= length  do
        local value
        
        if DATA_TYPE == "int" then
            -- 解析为32位有符号整数
            value = (data[index] << 24) | 
                   (data[index + 1] << 16) |
                   (data[index + 2] << 8) |
                   data[index + 3]
            -- 转换为有符号数
            if value >= 0x80000000 then
                value = value - 0x100000000
            end
            print(string.format("Lua: value=%d", value))
        else
            -- 解析为32位浮点数（小端序）
            local bytes = string.char(
                data[index],      -- 最低字节在前
                data[index + 1],
                data[index + 2],
                data[index + 3]   -- 最高字节在后
            )
            value = string.unpack("<f", bytes)
            print(string.format("Lua: value=%f", value))
        end
        
        table.insert(points, value)
        index = index + BYTES_PER_POINT
    end
    
    if #points > 0 then
        return {channel = channel, points = points}
    else
        return nil
    end
end