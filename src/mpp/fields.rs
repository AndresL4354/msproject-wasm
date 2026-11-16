// ARCHIVO GENERADO (gen_fields.py). No editar a mano.
// Identificadores de campo de Microsoft Project (formato MPP14): tipo de dato, id canonico y nombre.
#![allow(dead_code)]

pub const DT_NONE: u8 = 0;
pub const DT_STRING: u8 = 1;
pub const DT_DATE: u8 = 2;
pub const DT_NUMERIC: u8 = 3;
pub const DT_CURRENCY: u8 = 4;
pub const DT_BOOLEAN: u8 = 5;
pub const DT_DURATION: u8 = 6;
pub const DT_TIME_UNITS: u8 = 7;
pub const DT_WORK: u8 = 8;
pub const DT_INTEGER: u8 = 9;
pub const DT_SHORT: u8 = 10;
pub const DT_PERCENTAGE: u8 = 11;
pub const DT_UNITS: u8 = 12;
pub const DT_NOTES: u8 = 13;
pub const DT_CONSTRAINT: u8 = 14;
pub const DT_PRIORITY: u8 = 15;
pub const DT_TASK_TYPE: u8 = 16;
pub const DT_ACCRUE: u8 = 17;
pub const DT_RATE: u8 = 18;
pub const DT_RATE_UNITS: u8 = 19;
pub const DT_DELAY: u8 = 20;
pub const DT_WORK_UNITS: u8 = 21;
pub const DT_WORKGROUP: u8 = 22;
pub const DT_GUID: u8 = 23;
pub const DT_EARNED_VALUE_METHOD: u8 = 24;
pub const DT_RESOURCE_REQUEST_TYPE: u8 = 25;
pub const DT_BOOKING_TYPE: u8 = 26;
pub const DT_OTHER: u8 = 27;

pub struct FieldTable {
    pub base: u32,
    pub dt: &'static [u8],
    pub canon: &'static [u16],
    pub names: &'static [(u16, &'static str)],
    pub units: &'static [(u16, u16)],
}

pub static TASK: FieldTable = FieldTable {
    base: 0x0B400000,
    dt: &[8,8,8,6,8,4,4,4,4,4,4,4,4,4,1,13,1,14,2,5,6,6,6,9,5,15,1,6,6,6,6,6,11,11,5,2,2,2,2,2,2,2,2,2,2,6,6,27,27,1,1,1,2,2,1,2,2,1,2,2,1,2,2,1,2,2,1,1,1,1,1,5,5,5,5,5,5,5,5,5,5,5,5,4,1,10,9,3,3,3,3,3,5,2,13,1,1,3,5,2,2,2,1,6,6,6,4,4,4,5,5,5,1,1,0,0,0,0,0,2,4,0,0,0,0,0,0,0,16,5,0,0,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,9,0,0,6,8,8,6,0,4,4,4,0,0,0,0,0,0,0,7,7,0,7,0,7,7,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,17,0,5,27,0,1,0,0,0,0,0,0,0,0,0,27,0,1,1,1,1,0,0,0,5,5,0,0,0,0,0,0,5,0,0,0,0,0,0,0,0,0,9,0,0,0,5,0,0,10,5,5,5,5,0,9,27,0,4,4,4,4,4,4,4,2,2,2,2,2,2,2,2,2,2,6,6,6,6,6,6,6,2,2,2,2,2,2,2,2,2,2,5,5,5,5,5,5,5,5,5,5,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,7,7,7,7,7,7,7,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,9,0,0,0,0,0,1,1,1,2,2,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,5,0,0,5,0,9,1,5,5,5,5,5,5,5,5,5,5,5,0,0,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,0,2,6,6,0,4,0,0,0,0,1,0,0,27,27,1,1,0,0,0,0,0,9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,27,2,2,4,8,0,6,7,4,0,0,0,2,2,4,8,0,6,7,4,0,0,0,2,2,4,8,0,6,7,4,0,0,0,2,2,4,8,0,6,7,4,0,0,0,2,2,4,8,0,6,7,4,0,0,0,3,3,11,11,4,3,1,2,2,4,8,0,6,7,4,0,0,0,2,2,4,8,0,6,7,4,0,0,0,2,2,4,8,0,6,7,4,0,0,0,2,2,4,8,0,6,7,4,0,0,0,2,2,4,8,0,6,7,4,0,0,0,4,4,4,4,4,4,4,4,4,4,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,6,6,6,6,6,6,6,6,6,6,7,7,7,7,7,7,7,7,7,7,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,5,5,5,5,5,5,5,5,5,5,4,4,4,4,4,4,4,4,4,4,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,6,6,6,6,6,6,6,6,6,6,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,11,1,1,24,0,0,0,0,0,0,1,1,1,1,1,1,1,1,1,1,8,8,0,0,23,23,0,23,1,0,0,0,0,2,2,0,0,0,0,0,0,0,0,0,0,0,5,1,1,0,23,1,8,4,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,17,2,2,8,4,0,0,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,5,27,5,5,2,2,1,1,1,6,7,0,0,0,0,0,5,5,5,0,2,2,6,2,2,6,2,2,6,2,2,6,2,2,6,2,2,6,2,2,6,2,2,6,2,2,6,2,2,6,2,2,6,5,0,0,12,0,0,2,2,6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5,5,5,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,1,2,2,9,9],
    canon: &[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,94,16,17,18,19,20,21,22,23,24,25,26,65535,28,1340,30,31,32,33,34,1338,1339,37,38,39,40,41,42,65535,65535,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,249,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,202,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,181,182,183,184,185,186,187,188,189,190,191,192,193,194,195,196,197,198,199,200,201,202,203,204,205,206,207,208,209,210,211,212,213,214,215,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230,231,232,233,234,235,236,237,238,239,240,241,242,243,244,245,246,247,248,249,250,251,252,253,254,255,256,257,258,259,260,261,262,263,264,265,266,267,268,269,270,271,272,273,274,275,276,277,278,279,280,281,282,283,284,285,286,287,288,289,290,291,292,293,294,295,296,297,298,299,300,301,302,303,304,305,306,307,308,309,310,311,312,313,314,315,316,317,318,319,320,321,322,323,324,325,326,327,328,329,330,331,332,333,334,335,336,337,338,339,340,341,342,343,344,345,346,347,348,349,350,351,352,353,354,355,356,357,358,359,360,361,362,363,364,365,366,367,368,369,370,371,387,373,374,375,376,377,378,379,380,381,382,383,384,385,386,387,388,389,390,391,392,393,394,395,396,397,398,399,400,401,402,403,404,405,406,407,408,409,410,411,412,413,414,415,416,417,418,419,420,421,422,423,424,425,426,427,428,429,430,431,432,433,434,435,436,437,438,439,440,441,442,443,444,445,446,447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,466,467,468,469,470,471,472,473,474,475,476,477,478,479,480,481,65535,65535,484,485,486,65535,488,489,490,491,492,65535,65535,495,496,497,65535,499,500,501,502,503,65535,65535,506,507,508,65535,510,511,512,513,514,65535,65535,517,518,519,65535,521,522,523,524,525,65535,65535,528,529,530,65535,532,533,534,535,536,537,538,539,540,541,542,543,65535,65535,546,547,548,65535,550,551,552,553,554,65535,65535,557,558,559,65535,561,562,563,564,565,65535,65535,568,569,570,65535,572,573,574,575,576,65535,65535,579,580,581,65535,583,584,585,586,587,65535,65535,590,591,592,65535,594,595,596,597,598,599,600,601,602,603,604,605,606,607,608,609,610,611,612,613,614,615,616,617,618,619,620,621,622,623,624,625,626,627,628,629,630,631,632,633,634,635,636,637,638,639,640,641,642,643,644,645,646,647,648,649,650,651,652,653,654,655,656,657,658,659,660,661,662,663,664,665,666,667,668,669,670,671,672,673,674,675,676,677,678,679,680,681,682,683,684,685,686,687,688,689,690,691,692,693,694,695,696,697,698,699,700,701,702,703,704,705,706,707,708,709,710,711,712,713,714,715,716,717,718,719,720,721,722,723,724,725,726,727,728,729,730,731,732,733,734,735,736,737,738,739,740,741,742,743,744,745,746,747,748,749,750,751,752,753,754,755,756,757,758,759,760,761,762,763,764,765,766,767,768,769,770,771,772,773,774,775,776,777,778,779,780,781,782,783,784,785,786,787,788,789,790,791,792,793,794,795,796,797,798,799,800,801,802,803,804,805,806,807,808,809,810,811,812,813,814,815,816,817,818,819,820,821,822,823,824,825,826,827,828,829,830,831,832,833,834,835,836,837,838,839,840,841,842,843,844,845,846,847,848,849,850,851,852,853,854,855,856,857,858,859,860,861,862,863,864,865,866,867,868,869,870,871,872,873,874,875,876,877,878,879,880,881,882,883,884,885,886,887,888,889,890,891,892,893,894,895,896,897,898,899,900,901,902,903,904,905,906,907,908,909,910,911,912,913,914,915,916,917,918,919,920,921,922,923,924,925,926,927,928,929,930,931,932,933,934,935,936,937,938,939,940,941,942,943,944,945,946,947,948,949,950,951,952,953,954,955,956,957,958,959,960,961,962,963,964,965,966,967,968,969,970,971,972,973,974,975,976,977,978,979,980,981,982,983,984,985,986,987,988,989,990,991,992,993,994,995,996,997,998,999,1000,1001,1002,1003,1004,1005,1006,1007,1008,1009,1010,1011,1012,1013,1014,1015,1016,1017,1018,1019,1020,1021,1022,1023,1024,1025,1026,1027,1028,1029,1030,1031,1032,1033,1034,1035,1036,1037,1038,1039,1040,1041,1042,1043,1044,1045,1046,1047,1048,1049,1050,1051,1052,1053,1054,1055,1056,1057,1058,1059,1060,1061,1062,1063,1064,1065,1066,1067,1068,1069,1070,1071,1072,1073,1074,1075,1076,1077,1078,1079,1080,1081,1082,1083,1084,1085,1086,1087,1088,1089,1090,1091,1092,1093,1094,1095,1096,1097,1098,1099,1100,1101,1102,1103,1104,1105,1106,1107,1108,1109,1110,1111,1112,1113,1114,1115,1116,1117,1118,1119,1120,1121,1122,1123,1124,1125,1126,1127,1128,1129,1130,1131,1132,1133,1134,1135,1136,1137,1138,1139,1140,1141,1142,1143,1144,1145,1146,1147,1148,1149,1150,1151,1152,1153,1154,1155,1156,1157,1158,1159,1160,1161,1162,1163,1164,1165,1166,1167,1168,1169,1170,1171,1172,1173,1174,1175,1176,1177,1178,1179,1180,1181,1182,1183,1184,1185,1186,1187,1188,1189,1190,1191,1192,1193,1194,1195,1196,1197,1198,1199,1200,1201,1202,1203,1204,1205,1206,1207,1208,1209,1210,1211,1212,1213,1214,1215,1216,1217,1218,1219,1220,1221,1222,1223,1224,1225,1226,1227,1228,1229,1230,1231,1232,1233,1234,1235,1236,1237,1238,1239,1240,1241,1242,1243,1244,1245,1246,1247,1248,1249,1250,1251,1252,1253,1254,1255,1256,1257,1258,1259,1260,1261,1262,1263,1264,1265,1266,1267,1268,1269,1270,1271,1272,1273,1274,1275,1276,1277,1278,1279,1280,1281,1282,1283,1284,1285,1286,1287,1288,1289,1290,1291,1292,1293,1294,1295,1296,1297,1298,1299,1300,1301,1302,1303,1304,1305,1306,1307,1308,1309,1310,1311,1312,1313,1314,1315,1316,1317,1318,1319,1320,1321,1322,1323,1324,1325,1326,1327,1328,1329,1330,1331,1332,1333,1334,1335,1336,1337,1338,1339,1340,1341,1342,1343,1344,1345,1346,1347,1348,1349,1350,1351,1352,1353,1354,1355,1356,1357,1358,1359,1360,1361,1362,1363,1364,1365,1366,1367,1368,1369,1370,1371,1372,1373,1374,1375,1376,1377,1378,1379,1380,1381,1382,1383,1384,1385,1386,1387,1388,1389,1390,1391,1392,1393,1394,1395,1396,1397,1398,1399,1400,1401,1402,1403,1404,1405,1406,1407,1408,1409,1410,1411,1412,1413],
    names: &[
        (0, "Work"),
        (1, "Baseline Work"),
        (2, "Actual Work"),
        (3, "Work Variance"),
        (4, "Remaining Work"),
        (5, "Cost"),
        (6, "Baseline Cost"),
        (7, "Actual Cost"),
        (8, "Fixed Cost"),
        (9, "Cost Variance"),
        (10, "Remaining Cost"),
        (11, "BCWP"),
        (12, "BCWS"),
        (13, "SV"),
        (14, "Task Name"),
        (16, "WBS"),
        (17, "Constraint Type"),
        (18, "Constraint Date"),
        (19, "Critical"),
        (20, "Leveling Delay"),
        (21, "Free Slack"),
        (22, "Total Slack"),
        (23, "ID"),
        (24, "Milestone"),
        (25, "Priority"),
        (26, "Subproject File"),
        (28, "Actual Duration"),
        (30, "Duration Variance"),
        (31, "Remaining Duration"),
        (32, "% Complete"),
        (33, "% Work Complete"),
        (34, "Fixed Duration"),
        (37, "Early Start"),
        (38, "Early Finish"),
        (39, "Late Start"),
        (40, "Late Finish"),
        (41, "Actual Start"),
        (42, "Actual Finish"),
        (45, "Start Variance"),
        (46, "Finish Variance"),
        (47, "Predecessors"),
        (48, "Successors"),
        (49, "Resource Names"),
        (50, "Resource Initials"),
        (51, "Text1"),
        (52, "Start1"),
        (53, "Finish1"),
        (54, "Text2"),
        (55, "Start2"),
        (56, "Finish2"),
        (57, "Text3"),
        (58, "Start3"),
        (59, "Finish3"),
        (60, "Text4"),
        (61, "Start4"),
        (62, "Finish4"),
        (63, "Text5"),
        (64, "Start5"),
        (65, "Finish5"),
        (66, "Text6"),
        (67, "Text7"),
        (68, "Text8"),
        (69, "Text9"),
        (70, "Text10"),
        (71, "Marked"),
        (72, "Flag1"),
        (73, "Flag2"),
        (74, "Flag3"),
        (75, "Flag4"),
        (76, "Flag5"),
        (77, "Flag6"),
        (78, "Flag7"),
        (79, "Flag8"),
        (80, "Flag9"),
        (81, "Flag10"),
        (82, "Rollup"),
        (83, "CV"),
        (84, "Project"),
        (86, "Unique ID"),
        (87, "Number1"),
        (88, "Number2"),
        (89, "Number3"),
        (90, "Number4"),
        (91, "Number5"),
        (92, "Summary"),
        (93, "Created"),
        (94, "Notes"),
        (95, "Unique ID Predecessors"),
        (96, "Unique ID Successors"),
        (97, "Objects"),
        (98, "Linked Fields"),
        (99, "Resume"),
        (100, "Stop"),
        (101, "Resume No Earlier Than"),
        (102, "Outline Number"),
        (103, "Duration1"),
        (104, "Duration2"),
        (105, "Duration3"),
        (106, "Cost1"),
        (107, "Cost2"),
        (108, "Cost3"),
        (109, "Hide Bar"),
        (110, "Confirmed"),
        (111, "Update Needed"),
        (112, "Contact"),
        (113, "Resource Group"),
        (119, "Complete Through"),
        (120, "ACWP"),
        (128, "Type"),
        (132, "Effort Driven"),
        (152, "Duration Units"),
        (160, "Parent Task Unique ID"),
        (163, "Overtime Work"),
        (164, "Actual Overtime Work"),
        (165, "Remaining Overtime Work"),
        (166, "Regular Work"),
        (168, "Overtime Cost"),
        (169, "Actual Overtime Cost"),
        (170, "Remaining Overtime Cost"),
        (178, "Leveling Delay Units"),
        (179, "Baseline Duration Units"),
        (181, "Actual Duration Units"),
        (183, "Duration1 Units"),
        (184, "Duration2 Units"),
        (185, "Duration3 Units"),
        (200, "Fixed Cost Accrual"),
        (202, "Recurring"),
        (205, "Indicators"),
        (217, "Hyperlink"),
        (218, "Hyperlink Address"),
        (219, "Hyperlink SubAddress"),
        (220, "Hyperlink Href"),
        (224, "Assignment"),
        (225, "Overallocated"),
        (232, "External Task"),
        (242, "Subproject Task Unique ID"),
        (246, "Subproject Read Only"),
        (249, "Outline Level"),
        (250, "Response Pending"),
        (251, "TeamStatus Pending"),
        (252, "Leveling Can Split"),
        (253, "Level Assignments"),
        (255, "Subproject Task ID"),
        (256, "Work Contour"),
        (258, "Cost4"),
        (259, "Cost5"),
        (260, "Cost6"),
        (261, "Cost7"),
        (262, "Cost8"),
        (263, "Cost9"),
        (264, "Cost10"),
        (265, "Date1"),
        (266, "Date2"),
        (267, "Date3"),
        (268, "Date4"),
        (269, "Date5"),
        (270, "Date6"),
        (271, "Date7"),
        (272, "Date8"),
        (273, "Date9"),
        (274, "Date10"),
        (275, "Duration4"),
        (276, "Duration5"),
        (277, "Duration6"),
        (278, "Duration7"),
        (279, "Duration8"),
        (280, "Duration9"),
        (281, "Duration10"),
        (282, "Start6"),
        (283, "Finish6"),
        (284, "Start7"),
        (285, "Finish7"),
        (286, "Start8"),
        (287, "Finish8"),
        (288, "Start9"),
        (289, "Finish9"),
        (290, "Start10"),
        (291, "Finish10"),
        (292, "Flag11"),
        (293, "Flag12"),
        (294, "Flag13"),
        (295, "Flag14"),
        (296, "Flag15"),
        (297, "Flag16"),
        (298, "Flag17"),
        (299, "Flag18"),
        (300, "Flag19"),
        (301, "Flag20"),
        (302, "Number6"),
        (303, "Number7"),
        (304, "Number8"),
        (305, "Number9"),
        (306, "Number10"),
        (307, "Number11"),
        (308, "Number12"),
        (309, "Number13"),
        (310, "Number14"),
        (311, "Number15"),
        (312, "Number16"),
        (313, "Number17"),
        (314, "Number18"),
        (315, "Number19"),
        (316, "Number20"),
        (317, "Text11"),
        (318, "Text12"),
        (319, "Text13"),
        (320, "Text14"),
        (321, "Text15"),
        (322, "Text16"),
        (323, "Text17"),
        (324, "Text18"),
        (325, "Text19"),
        (326, "Text20"),
        (327, "Text21"),
        (328, "Text22"),
        (329, "Text23"),
        (330, "Text24"),
        (331, "Text25"),
        (332, "Text26"),
        (333, "Text27"),
        (334, "Text28"),
        (335, "Text29"),
        (336, "Text30"),
        (337, "Duration4 Units"),
        (338, "Duration5 Units"),
        (339, "Duration6 Units"),
        (340, "Duration7 Units"),
        (341, "Duration8 Units"),
        (342, "Duration9 Units"),
        (343, "Duration10 Units"),
        (349, "Resource Phonetics"),
        (360, "Index"),
        (366, "Assignment Delay"),
        (367, "Assignment Units"),
        (368, "Cost Rate Table"),
        (369, "Preleveled Start"),
        (370, "Preleveled Finish"),
        (387, "Summary Progress"),
        (396, "Estimated"),
        (399, "Ignore Resource Calendar"),
        (401, "Calendar Unique ID"),
        (402, "Task Calendar"),
        (403, "Duration1 Estimated"),
        (404, "Duration2 Estimated"),
        (405, "Duration3 Estimated"),
        (406, "Duration4 Estimated"),
        (407, "Duration5 Estimated"),
        (408, "Duration6 Estimated"),
        (409, "Duration7 Estimated"),
        (410, "Duration8 Estimated"),
        (411, "Duration9 Estimated"),
        (412, "Duration10 Estimated"),
        (413, "Baseline Duration Estimated"),
        (416, "Outline Code1"),
        (418, "Outline Code2"),
        (420, "Outline Code3"),
        (422, "Outline Code4"),
        (424, "Outline Code5"),
        (426, "Outline Code6"),
        (428, "Outline Code7"),
        (430, "Outline Code8"),
        (432, "Outline Code9"),
        (434, "Outline Code10"),
        (437, "Deadline"),
        (438, "Start Slack"),
        (439, "Finish Slack"),
        (441, "VAC"),
        (446, "Group By Summary"),
        (449, "WBS Predecessors"),
        (450, "WBS Successors"),
        (451, "Resource Type"),
        (452, "Hyperlink Screen Tip"),
        (458, "Subproject Tasks Unique ID Offset"),
        (480, "Baseline Fixed Cost"),
        (537, "CPI"),
        (538, "SPI"),
        (539, "CV%"),
        (540, "SV%"),
        (541, "EAC"),
        (542, "TCPI"),
        (543, "Status"),
        (1119, "Physical % Complete"),
        (1120, "Request/Demand"),
        (1121, "Status Indicator"),
        (1122, "Earned Value Method"),
        (1139, "Actual Work Protected"),
        (1140, "Actual Overtime Work Protected"),
        (1147, "Deliverable Type"),
        (1152, "Deliverable Start"),
        (1153, "Deliverable Finish"),
        (1165, "Publish"),
        (1166, "Status Manager"),
        (1167, "Error Message"),
        (1170, "Assignment Owner"),
        (1171, "Budget Work"),
        (1172, "Budget Cost"),
        (1173, "Baseline Fixed Cost Accrual"),
        (1174, "Baseline Deliverable Start"),
        (1175, "Baseline Deliverable Finish"),
        (1176, "Baseline Budget Work"),
        (1177, "Baseline Budget Cost"),
        (1250, "Recalc Outline Codes"),
        (1276, "Deliverable Name"),
        (1279, "Active"),
        (1280, "Task Mode"),
        (1281, "Placeholder"),
        (1282, "Warning"),
        (1283, "Start"),
        (1284, "Finish"),
        (1285, "Start Text"),
        (1286, "Finish Text"),
        (1287, "Duration Text"),
        (1288, "Manual Duration"),
        (1289, "Manual Duration Units"),
        (1295, "Is Start Valid"),
        (1296, "Is Finish Valid"),
        (1297, "Is Duration Valid"),
        (1299, "Baseline Start"),
        (1300, "Baseline Finish"),
        (1301, "Baseline Duration"),
        (1332, "Ignore Warnings"),
        (1335, "Peak"),
        (1338, "Scheduled Start"),
        (1339, "Scheduled Finish"),
        (1340, "Scheduled Duration"),
        (1381, "Path Driving Predecessor"),
        (1382, "Path Predecessor"),
        (1383, "Path Driven Successor"),
        (1384, "Path Successor"),
        (1405, "Task Summary Name"),
        (1407, "Board Status"),
        (1408, "Show On Board"),
        (1409, "Sprint"),
        (1410, "Sprint Start"),
        (1411, "Sprint Finish"),
        (1412, "Board Status ID"),
        (1413, "Sprint ID"),
        (65535, "Baseline Estimated Duration"),
    ],
    units: &[(20,178),(21,181),(65535,179),(28,181),(1340,181),(31,181),(103,183),(104,184),(105,185),(275,337),(276,338),(277,339),(278,340),(279,341),(280,342),(281,343),(438,181),(439,181),(639,649),(640,650),(641,651),(642,652),(643,653),(644,654),(645,655),(646,656),(647,657),(648,658),(1288,1289),(1301,179),(1304,488),(1307,499),(1310,510),(1313,521),(1316,532),(1319,550),(1322,561),(1325,572),(1328,583),(1331,594)],
};

pub mod task {
    pub const ACTIVE: u16 = 1279;
    pub const ACTUAL_COST: u16 = 7;
    pub const ACTUAL_DURATION: u16 = 28;
    pub const ACTUAL_DURATION_UNITS: u16 = 181;
    pub const ACTUAL_FINISH: u16 = 42;
    pub const ACTUAL_OVERTIME_COST: u16 = 169;
    pub const ACTUAL_OVERTIME_WORK: u16 = 164;
    pub const ACTUAL_OVERTIME_WORK_PROTECTED: u16 = 1140;
    pub const ACTUAL_START: u16 = 41;
    pub const ACTUAL_WORK: u16 = 2;
    pub const ACTUAL_WORK_PROTECTED: u16 = 1139;
    pub const ACWP: u16 = 120;
    pub const ASSIGNMENT: u16 = 224;
    pub const ASSIGNMENT_DELAY: u16 = 366;
    pub const ASSIGNMENT_OWNER: u16 = 1170;
    pub const ASSIGNMENT_UNITS: u16 = 367;
    pub const BASELINE_BUDGET_COST: u16 = 1177;
    pub const BASELINE_BUDGET_WORK: u16 = 1176;
    pub const BASELINE_COST: u16 = 6;
    pub const BASELINE_DELIVERABLE_FINISH: u16 = 1175;
    pub const BASELINE_DELIVERABLE_START: u16 = 1174;
    pub const BASELINE_DURATION: u16 = 1301;
    pub const BASELINE_DURATION_ESTIMATED: u16 = 413;
    pub const BASELINE_DURATION_UNITS: u16 = 179;
    pub const BASELINE_ESTIMATED_DURATION: u16 = 65535;
    pub const BASELINE_FINISH: u16 = 1300;
    pub const BASELINE_FIXED_COST: u16 = 480;
    pub const BASELINE_FIXED_COST_ACCRUAL: u16 = 1173;
    pub const BASELINE_START: u16 = 1299;
    pub const BASELINE_WORK: u16 = 1;
    pub const BCWP: u16 = 11;
    pub const BCWS: u16 = 12;
    pub const BOARD_STATUS: u16 = 1407;
    pub const BOARD_STATUS_ID: u16 = 1412;
    pub const BUDGET_COST: u16 = 1172;
    pub const BUDGET_WORK: u16 = 1171;
    pub const CALENDAR_UNIQUE_ID: u16 = 401;
    pub const COMPLETE_THROUGH: u16 = 119;
    pub const CONFIRMED: u16 = 110;
    pub const CONSTRAINT_DATE: u16 = 18;
    pub const CONSTRAINT_TYPE: u16 = 17;
    pub const CONTACT: u16 = 112;
    pub const COST: u16 = 5;
    pub const COST1: u16 = 106;
    pub const COST10: u16 = 264;
    pub const COST2: u16 = 107;
    pub const COST3: u16 = 108;
    pub const COST4: u16 = 258;
    pub const COST5: u16 = 259;
    pub const COST6: u16 = 260;
    pub const COST7: u16 = 261;
    pub const COST8: u16 = 262;
    pub const COST9: u16 = 263;
    pub const COST_RATE_TABLE: u16 = 368;
    pub const COST_VARIANCE: u16 = 9;
    pub const CPI: u16 = 537;
    pub const CREATED: u16 = 93;
    pub const CRITICAL: u16 = 19;
    pub const CV: u16 = 83;
    pub const CVPERCENT: u16 = 539;
    pub const DATE1: u16 = 265;
    pub const DATE10: u16 = 274;
    pub const DATE2: u16 = 266;
    pub const DATE3: u16 = 267;
    pub const DATE4: u16 = 268;
    pub const DATE5: u16 = 269;
    pub const DATE6: u16 = 270;
    pub const DATE7: u16 = 271;
    pub const DATE8: u16 = 272;
    pub const DATE9: u16 = 273;
    pub const DEADLINE: u16 = 437;
    pub const DELIVERABLE_FINISH: u16 = 1153;
    pub const DELIVERABLE_NAME: u16 = 1276;
    pub const DELIVERABLE_START: u16 = 1152;
    pub const DELIVERABLE_TYPE: u16 = 1147;
    pub const DURATION1: u16 = 103;
    pub const DURATION10: u16 = 281;
    pub const DURATION10_ESTIMATED: u16 = 412;
    pub const DURATION10_UNITS: u16 = 343;
    pub const DURATION1_ESTIMATED: u16 = 403;
    pub const DURATION1_UNITS: u16 = 183;
    pub const DURATION2: u16 = 104;
    pub const DURATION2_ESTIMATED: u16 = 404;
    pub const DURATION2_UNITS: u16 = 184;
    pub const DURATION3: u16 = 105;
    pub const DURATION3_ESTIMATED: u16 = 405;
    pub const DURATION3_UNITS: u16 = 185;
    pub const DURATION4: u16 = 275;
    pub const DURATION4_ESTIMATED: u16 = 406;
    pub const DURATION4_UNITS: u16 = 337;
    pub const DURATION5: u16 = 276;
    pub const DURATION5_ESTIMATED: u16 = 407;
    pub const DURATION5_UNITS: u16 = 338;
    pub const DURATION6: u16 = 277;
    pub const DURATION6_ESTIMATED: u16 = 408;
    pub const DURATION6_UNITS: u16 = 339;
    pub const DURATION7: u16 = 278;
    pub const DURATION7_ESTIMATED: u16 = 409;
    pub const DURATION7_UNITS: u16 = 340;
    pub const DURATION8: u16 = 279;
    pub const DURATION8_ESTIMATED: u16 = 410;
    pub const DURATION8_UNITS: u16 = 341;
    pub const DURATION9: u16 = 280;
    pub const DURATION9_ESTIMATED: u16 = 411;
    pub const DURATION9_UNITS: u16 = 342;
    pub const DURATION_TEXT: u16 = 1287;
    pub const DURATION_UNITS: u16 = 152;
    pub const DURATION_VARIANCE: u16 = 30;
    pub const EAC: u16 = 541;
    pub const EARLY_FINISH: u16 = 38;
    pub const EARLY_START: u16 = 37;
    pub const EARNED_VALUE_METHOD: u16 = 1122;
    pub const EFFORT_DRIVEN: u16 = 132;
    pub const ERROR_MESSAGE: u16 = 1167;
    pub const ESTIMATED: u16 = 396;
    pub const EXTERNAL_TASK: u16 = 232;
    pub const FINISH: u16 = 1284;
    pub const FINISH1: u16 = 53;
    pub const FINISH10: u16 = 291;
    pub const FINISH2: u16 = 56;
    pub const FINISH3: u16 = 59;
    pub const FINISH4: u16 = 62;
    pub const FINISH5: u16 = 65;
    pub const FINISH6: u16 = 283;
    pub const FINISH7: u16 = 285;
    pub const FINISH8: u16 = 287;
    pub const FINISH9: u16 = 289;
    pub const FINISH_SLACK: u16 = 439;
    pub const FINISH_TEXT: u16 = 1286;
    pub const FINISH_VARIANCE: u16 = 46;
    pub const FIXED_COST: u16 = 8;
    pub const FIXED_COST_ACCRUAL: u16 = 200;
    pub const FIXED_DURATION: u16 = 34;
    pub const FLAG1: u16 = 72;
    pub const FLAG10: u16 = 81;
    pub const FLAG11: u16 = 292;
    pub const FLAG12: u16 = 293;
    pub const FLAG13: u16 = 294;
    pub const FLAG14: u16 = 295;
    pub const FLAG15: u16 = 296;
    pub const FLAG16: u16 = 297;
    pub const FLAG17: u16 = 298;
    pub const FLAG18: u16 = 299;
    pub const FLAG19: u16 = 300;
    pub const FLAG2: u16 = 73;
    pub const FLAG20: u16 = 301;
    pub const FLAG3: u16 = 74;
    pub const FLAG4: u16 = 75;
    pub const FLAG5: u16 = 76;
    pub const FLAG6: u16 = 77;
    pub const FLAG7: u16 = 78;
    pub const FLAG8: u16 = 79;
    pub const FLAG9: u16 = 80;
    pub const FREE_SLACK: u16 = 21;
    pub const GROUP_BY_SUMMARY: u16 = 446;
    pub const HIDE_BAR: u16 = 109;
    pub const HYPERLINK: u16 = 217;
    pub const HYPERLINK_ADDRESS: u16 = 218;
    pub const HYPERLINK_HREF: u16 = 220;
    pub const HYPERLINK_SCREEN_TIP: u16 = 452;
    pub const HYPERLINK_SUBADDRESS: u16 = 219;
    pub const ID: u16 = 23;
    pub const IGNORE_RESOURCE_CALENDAR: u16 = 399;
    pub const IGNORE_WARNINGS: u16 = 1332;
    pub const INDEX: u16 = 360;
    pub const INDICATORS: u16 = 205;
    pub const IS_DURATION_VALID: u16 = 1297;
    pub const IS_FINISH_VALID: u16 = 1296;
    pub const IS_START_VALID: u16 = 1295;
    pub const LATE_FINISH: u16 = 40;
    pub const LATE_START: u16 = 39;
    pub const LEVELING_CAN_SPLIT: u16 = 252;
    pub const LEVELING_DELAY: u16 = 20;
    pub const LEVELING_DELAY_UNITS: u16 = 178;
    pub const LEVEL_ASSIGNMENTS: u16 = 253;
    pub const LINKED_FIELDS: u16 = 98;
    pub const MANUAL_DURATION: u16 = 1288;
    pub const MANUAL_DURATION_UNITS: u16 = 1289;
    pub const MARKED: u16 = 71;
    pub const MILESTONE: u16 = 24;
    pub const NAME: u16 = 14;
    pub const NOTES: u16 = 94;
    pub const NUMBER1: u16 = 87;
    pub const NUMBER10: u16 = 306;
    pub const NUMBER11: u16 = 307;
    pub const NUMBER12: u16 = 308;
    pub const NUMBER13: u16 = 309;
    pub const NUMBER14: u16 = 310;
    pub const NUMBER15: u16 = 311;
    pub const NUMBER16: u16 = 312;
    pub const NUMBER17: u16 = 313;
    pub const NUMBER18: u16 = 314;
    pub const NUMBER19: u16 = 315;
    pub const NUMBER2: u16 = 88;
    pub const NUMBER20: u16 = 316;
    pub const NUMBER3: u16 = 89;
    pub const NUMBER4: u16 = 90;
    pub const NUMBER5: u16 = 91;
    pub const NUMBER6: u16 = 302;
    pub const NUMBER7: u16 = 303;
    pub const NUMBER8: u16 = 304;
    pub const NUMBER9: u16 = 305;
    pub const OBJECTS: u16 = 97;
    pub const OUTLINE_CODE1: u16 = 416;
    pub const OUTLINE_CODE10: u16 = 434;
    pub const OUTLINE_CODE2: u16 = 418;
    pub const OUTLINE_CODE3: u16 = 420;
    pub const OUTLINE_CODE4: u16 = 422;
    pub const OUTLINE_CODE5: u16 = 424;
    pub const OUTLINE_CODE6: u16 = 426;
    pub const OUTLINE_CODE7: u16 = 428;
    pub const OUTLINE_CODE8: u16 = 430;
    pub const OUTLINE_CODE9: u16 = 432;
    pub const OUTLINE_LEVEL: u16 = 249;
    pub const OUTLINE_NUMBER: u16 = 102;
    pub const OVERALLOCATED: u16 = 225;
    pub const OVERTIME_COST: u16 = 168;
    pub const OVERTIME_WORK: u16 = 163;
    pub const PARENT_TASK_UNIQUE_ID: u16 = 160;
    pub const PATH_DRIVEN_SUCCESSOR: u16 = 1383;
    pub const PATH_DRIVING_PREDECESSOR: u16 = 1381;
    pub const PATH_PREDECESSOR: u16 = 1382;
    pub const PATH_SUCCESSOR: u16 = 1384;
    pub const PEAK: u16 = 1335;
    pub const PERCENT_COMPLETE: u16 = 32;
    pub const PERCENT_WORK_COMPLETE: u16 = 33;
    pub const PHYSICAL_PERCENT_COMPLETE: u16 = 1119;
    pub const PLACEHOLDER: u16 = 1281;
    pub const PREDECESSORS: u16 = 47;
    pub const PRELEVELED_FINISH: u16 = 370;
    pub const PRELEVELED_START: u16 = 369;
    pub const PRIORITY: u16 = 25;
    pub const PROJECT: u16 = 84;
    pub const PUBLISH: u16 = 1165;
    pub const RECALC_OUTLINE_CODES: u16 = 1250;
    pub const RECURRING: u16 = 202;
    pub const REGULAR_WORK: u16 = 166;
    pub const REMAINING_COST: u16 = 10;
    pub const REMAINING_DURATION: u16 = 31;
    pub const REMAINING_OVERTIME_COST: u16 = 170;
    pub const REMAINING_OVERTIME_WORK: u16 = 165;
    pub const REMAINING_WORK: u16 = 4;
    pub const REQUEST_DEMAND: u16 = 1120;
    pub const RESOURCE_GROUP: u16 = 113;
    pub const RESOURCE_INITIALS: u16 = 50;
    pub const RESOURCE_NAMES: u16 = 49;
    pub const RESOURCE_PHONETICS: u16 = 349;
    pub const RESOURCE_TYPE: u16 = 451;
    pub const RESPONSE_PENDING: u16 = 250;
    pub const RESUME: u16 = 99;
    pub const RESUME_NO_EARLIER_THAN: u16 = 101;
    pub const ROLLUP: u16 = 82;
    pub const SCHEDULED_DURATION: u16 = 1340;
    pub const SCHEDULED_FINISH: u16 = 1339;
    pub const SCHEDULED_START: u16 = 1338;
    pub const SHOW_ON_BOARD: u16 = 1408;
    pub const SPI: u16 = 538;
    pub const SPRINT: u16 = 1409;
    pub const SPRINT_FINISH: u16 = 1411;
    pub const SPRINT_ID: u16 = 1413;
    pub const SPRINT_START: u16 = 1410;
    pub const START: u16 = 1283;
    pub const START1: u16 = 52;
    pub const START10: u16 = 290;
    pub const START2: u16 = 55;
    pub const START3: u16 = 58;
    pub const START4: u16 = 61;
    pub const START5: u16 = 64;
    pub const START6: u16 = 282;
    pub const START7: u16 = 284;
    pub const START8: u16 = 286;
    pub const START9: u16 = 288;
    pub const START_SLACK: u16 = 438;
    pub const START_TEXT: u16 = 1285;
    pub const START_VARIANCE: u16 = 45;
    pub const STATUS: u16 = 543;
    pub const STATUS_INDICATOR: u16 = 1121;
    pub const STATUS_MANAGER: u16 = 1166;
    pub const STOP: u16 = 100;
    pub const SUBPROJECT_FILE: u16 = 26;
    pub const SUBPROJECT_READ_ONLY: u16 = 246;
    pub const SUBPROJECT_TASKS_UNIQUEID_OFFSET: u16 = 458;
    pub const SUBPROJECT_TASK_ID: u16 = 255;
    pub const SUBPROJECT_TASK_UNIQUE_ID: u16 = 242;
    pub const SUCCESSORS: u16 = 48;
    pub const SUMMARY: u16 = 92;
    pub const SUMMARY_PROGRESS: u16 = 387;
    pub const SV: u16 = 13;
    pub const SVPERCENT: u16 = 540;
    pub const TASK_CALENDAR: u16 = 402;
    pub const TASK_MODE: u16 = 1280;
    pub const TASK_SUMMARY_NAME: u16 = 1405;
    pub const TCPI: u16 = 542;
    pub const TEAMSTATUS_PENDING: u16 = 251;
    pub const TEXT1: u16 = 51;
    pub const TEXT10: u16 = 70;
    pub const TEXT11: u16 = 317;
    pub const TEXT12: u16 = 318;
    pub const TEXT13: u16 = 319;
    pub const TEXT14: u16 = 320;
    pub const TEXT15: u16 = 321;
    pub const TEXT16: u16 = 322;
    pub const TEXT17: u16 = 323;
    pub const TEXT18: u16 = 324;
    pub const TEXT19: u16 = 325;
    pub const TEXT2: u16 = 54;
    pub const TEXT20: u16 = 326;
    pub const TEXT21: u16 = 327;
    pub const TEXT22: u16 = 328;
    pub const TEXT23: u16 = 329;
    pub const TEXT24: u16 = 330;
    pub const TEXT25: u16 = 331;
    pub const TEXT26: u16 = 332;
    pub const TEXT27: u16 = 333;
    pub const TEXT28: u16 = 334;
    pub const TEXT29: u16 = 335;
    pub const TEXT3: u16 = 57;
    pub const TEXT30: u16 = 336;
    pub const TEXT4: u16 = 60;
    pub const TEXT5: u16 = 63;
    pub const TEXT6: u16 = 66;
    pub const TEXT7: u16 = 67;
    pub const TEXT8: u16 = 68;
    pub const TEXT9: u16 = 69;
    pub const TOTAL_SLACK: u16 = 22;
    pub const TYPE: u16 = 128;
    pub const UNIQUE_ID: u16 = 86;
    pub const UNIQUE_ID_PREDECESSORS: u16 = 95;
    pub const UNIQUE_ID_SUCCESSORS: u16 = 96;
    pub const UPDATE_NEEDED: u16 = 111;
    pub const VAC: u16 = 441;
    pub const WARNING: u16 = 1282;
    pub const WBS: u16 = 16;
    pub const WBS_PREDECESSORS: u16 = 449;
    pub const WBS_SUCCESSORS: u16 = 450;
    pub const WORK: u16 = 0;
    pub const WORK_CONTOUR: u16 = 256;
    pub const WORK_VARIANCE: u16 = 3;
}

pub static RESOURCE: FieldTable = FieldTable {
    base: 0x0C400000,
    dt: &[9,1,1,1,12,1,18,18,1,1,1,4,4,8,8,8,8,4,4,17,13,4,8,6,4,5,12,9,13,11,1,1,1,3,5,1,0,0,8,8,8,0,0,0,0,0,0,4,4,4,0,4,4,4,4,0,0,2,2,0,0,27,27,27,27,27,0,0,0,0,19,19,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,2,2,2,2,2,2,2,2,2,2,3,3,3,3,3,6,6,6,7,7,7,4,4,4,5,5,5,5,5,5,5,5,5,5,27,0,1,1,1,1,0,0,5,0,0,0,0,0,0,0,9,0,0,0,0,0,0,1,0,0,0,5,27,0,4,4,4,4,4,4,4,2,2,2,2,2,2,2,2,2,2,6,6,6,6,6,6,6,2,2,2,2,2,5,5,5,5,5,5,5,5,5,5,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,2,2,2,2,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,7,7,7,7,7,7,7,1,9,0,0,0,20,12,2,2,5,2,6,5,2,5,5,4,10,2,2,22,1,0,0,27,0,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,0,1,27,4,0,0,0,0,5,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,27,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,8,4,0,0,0,0,2,2,0,0,1,9,0,0,4,4,4,4,4,4,4,4,4,4,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,6,6,6,6,6,6,6,6,6,6,7,7,7,7,7,7,7,7,7,7,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,5,5,5,1,1,5,5,1,2,5,5,0,0,26,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,8,8,0,0,0,0,2,0,23,23,0,0,0,0,0,0,0,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0,5,8,4,5,8,4,0,0,8,4,0,0,8,4,0,0,8,4,0,0,8,4,0,0,8,4,0,0,8,4,0,0,8,4,0,0,8,4,0,0,8,4,0,0,8,4,0,0,5,1,0,1,0,0,0,0,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,0,2,0,0,12,0,0,1],
    canon: &[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,28,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,181,182,183,184,185,186,187,188,189,190,191,192,193,194,195,196,197,198,199,200,201,202,203,204,205,206,207,208,209,210,211,212,213,214,215,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230,231,232,233,234,235,236,237,238,239,240,241,242,243,244,245,246,247,248,249,250,251,252,253,254,255,256,257,258,259,260,261,262,263,264,265,266,267,268,269,270,271,272,273,274,275,276,277,278,279,280,281,282,283,284,285,286,287,288,289,290,291,292,293,294,295,296,297,298,299,300,301,302,303,304,305,306,307,308,309,310,311,312,313,314,315,316,317,318,319,320,321,322,323,324,325,326,327,328,329,330,331,332,333,334,335,336,337,338,339,340,341,342,343,344,345,346,347,348,349,350,351,352,353,354,355,356,357,358,359,360,361,362,363,364,365,366,367,368,369,370,371,372,373,374,375,376,377,378,379,380,381,382,383,384,385,386,387,388,389,390,391,392,393,394,395,396,397,398,399,400,401,402,403,404,405,406,407,408,409,410,411,412,413,414,415,416,417,418,419,420,421,422,423,424,425,426,427,428,429,430,431,432,433,434,435,436,437,438,439,440,441,442,443,444,445,446,447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,466,467,468,469,470,471,472,473,474,475,476,477,478,479,480,481,482,483,484,485,486,487,488,489,490,491,492,493,494,495,496,497,498,499,500,501,502,503,504,505,506,507,508,509,510,511,512,513,514,515,516,517,518,519,520,521,522,523,524,525,526,527,528,529,530,531,532,533,534,535,536,537,538,539,540,541,542,543,544,545,546,547,548,549,550,551,552,553,554,555,556,557,558,559,560,561,562,563,564,565,566,567,568,569,570,571,572,573,574,575,576,577,578,579,580,581,582,583,584,585,586,587,588,589,590,591,592,593,594,595,596,597,598,599,600,601,602,603,604,605,606,607,608,609,610,611,612,613,614,615,616,617,618,619,620,621,622,623,624,625,626,627,628,629,630,631,632,633,634,635,636,637,638,639,640,641,642,643,644,645,646,647,648,649,650,651,652,653,654,655,656,657,658,659,660,661,662,663,664,665,666,667,668,669,670,671,672,673,674,675,676,677,678,679,680,681,682,683,684,685,686,687,688,689,690,691,692,693,694,695,696,697,698,699,700,701,702,703,704,705,706,707,708,709,710,711,712,713,714,715,716,717,718,719,720,721,722,723,724,725,726,727,728,729,730,731,732,733,734,735,736,737,738,739,740,741,742,743,744,745,746,747,748,749,750,751,752,753,754,755,756,757,758,759,760,761,762,763,764,765,766,767,768,769,770,771,772,773,774,775,776,777,778,779,780,781,782,783,784,785,786,787,788,789,790,791,792,793,794,795,796,797,798,799,800,801,802,803,804,805,806,807,808,809,810,811,812,813,814,815,816,817,818,819,820,821,822,823,824,825,826,827,828,829,830,831,832,833,834,835,836,837,838,839,840,841,842,843,844,845,846,847,848,849,850,851,852,853,854,855,856,857,858,859,860,861,862],
    names: &[
        (0, "ID"),
        (1, "Name"),
        (2, "Initials"),
        (3, "Group"),
        (4, "Max Units"),
        (5, "Base Calendar"),
        (6, "Standard Rate"),
        (7, "Overtime Rate"),
        (8, "Text1"),
        (9, "Text2"),
        (10, "Code"),
        (11, "Actual Cost"),
        (12, "Cost"),
        (13, "Work"),
        (14, "Actual Work"),
        (15, "Baseline Work"),
        (16, "Overtime Work"),
        (17, "Baseline Cost"),
        (18, "Cost Per Use"),
        (19, "Accrue At"),
        (21, "Remaining Cost"),
        (22, "Remaining Work"),
        (23, "Work Variance"),
        (24, "Cost Variance"),
        (25, "Overallocated"),
        (26, "Peak"),
        (27, "Unique ID"),
        (28, "Notes"),
        (29, "% Work Complete"),
        (30, "Text3"),
        (31, "Text4"),
        (32, "Text5"),
        (33, "Objects"),
        (34, "Linked Fields"),
        (35, "Email Address"),
        (38, "Regular Work"),
        (39, "Actual Overtime Work"),
        (40, "Remaining Overtime Work"),
        (47, "Overtime Cost"),
        (48, "Actual Overtime Cost"),
        (49, "Remaining Overtime Cost"),
        (51, "BCWS"),
        (52, "BCWP"),
        (53, "ACWP"),
        (54, "SV"),
        (57, "Available From"),
        (58, "Available To"),
        (70, "Standard Rate Units"),
        (71, "Overtime Rate Units"),
        (86, "Indicators"),
        (97, "Text6"),
        (98, "Text7"),
        (99, "Text8"),
        (100, "Text9"),
        (101, "Text10"),
        (102, "Start1"),
        (103, "Start2"),
        (104, "Start3"),
        (105, "Start4"),
        (106, "Start5"),
        (107, "Finish1"),
        (108, "Finish2"),
        (109, "Finish3"),
        (110, "Finish4"),
        (111, "Finish5"),
        (112, "Number1"),
        (113, "Number2"),
        (114, "Number3"),
        (115, "Number4"),
        (116, "Number5"),
        (117, "Duration1"),
        (118, "Duration2"),
        (119, "Duration3"),
        (120, "Duration1 Units"),
        (121, "Duration2 Units"),
        (122, "Duration3 Units"),
        (123, "Cost1"),
        (124, "Cost2"),
        (125, "Cost3"),
        (126, "Flag10"),
        (127, "Flag1"),
        (128, "Flag2"),
        (129, "Flag3"),
        (130, "Flag4"),
        (131, "Flag5"),
        (132, "Flag6"),
        (133, "Flag7"),
        (134, "Flag8"),
        (135, "Flag9"),
        (138, "Hyperlink"),
        (139, "Hyperlink Address"),
        (140, "Hyperlink SubAddress"),
        (141, "Hyperlink Href"),
        (144, "Assignment"),
        (152, "Subproject Unique Resource ID"),
        (159, "Task Summary Name"),
        (163, "Can Level"),
        (164, "Work Contour"),
        (166, "Cost4"),
        (167, "Cost5"),
        (168, "Cost6"),
        (169, "Cost7"),
        (170, "Cost8"),
        (171, "Cost9"),
        (172, "Cost10"),
        (173, "Date1"),
        (174, "Date2"),
        (175, "Date3"),
        (176, "Date4"),
        (177, "Date5"),
        (178, "Date6"),
        (179, "Date7"),
        (180, "Date8"),
        (181, "Date9"),
        (182, "Date10"),
        (183, "Duration4"),
        (184, "Duration5"),
        (185, "Duration6"),
        (186, "Duration7"),
        (187, "Duration8"),
        (188, "Duration9"),
        (189, "Duration10"),
        (190, "Finish6"),
        (191, "Finish7"),
        (192, "Finish8"),
        (193, "Finish9"),
        (194, "Finish10"),
        (195, "Flag11"),
        (196, "Flag12"),
        (197, "Flag13"),
        (198, "Flag14"),
        (199, "Flag15"),
        (200, "Flag16"),
        (201, "Flag17"),
        (202, "Flag18"),
        (203, "Flag19"),
        (204, "Flag20"),
        (205, "Number6"),
        (206, "Number7"),
        (207, "Number8"),
        (208, "Number9"),
        (209, "Number10"),
        (210, "Number11"),
        (211, "Number12"),
        (212, "Number13"),
        (213, "Number14"),
        (214, "Number15"),
        (215, "Number16"),
        (216, "Number17"),
        (217, "Number18"),
        (218, "Number19"),
        (219, "Number20"),
        (220, "Start6"),
        (221, "Start7"),
        (222, "Start8"),
        (223, "Start9"),
        (224, "Start10"),
        (225, "Text11"),
        (226, "Text12"),
        (227, "Text13"),
        (228, "Text14"),
        (229, "Text15"),
        (230, "Text16"),
        (231, "Text17"),
        (232, "Text18"),
        (233, "Text19"),
        (234, "Text20"),
        (235, "Text21"),
        (236, "Text22"),
        (237, "Text23"),
        (238, "Text24"),
        (239, "Text25"),
        (240, "Text26"),
        (241, "Text27"),
        (242, "Text28"),
        (243, "Text29"),
        (244, "Text30"),
        (245, "Duration4 Units"),
        (246, "Duration5 Units"),
        (247, "Duration6 Units"),
        (248, "Duration7 Units"),
        (249, "Duration8 Units"),
        (250, "Duration9 Units"),
        (251, "Duration10 Units"),
        (252, "Phonetics"),
        (253, "Index"),
        (257, "Assignment Delay"),
        (258, "Assignment Units"),
        (259, "Baseline Start"),
        (260, "Baseline Finish"),
        (261, "Confirmed"),
        (262, "Finish"),
        (263, "Leveling Delay"),
        (264, "Response Pending"),
        (265, "Start"),
        (266, "TeamStatus Pending"),
        (267, "Update Needed"),
        (268, "CV"),
        (269, "Cost Rate Table"),
        (270, "Actual Start"),
        (271, "Actual Finish"),
        (272, "Workgroup"),
        (273, "Project"),
        (278, "Outline Code1"),
        (280, "Outline Code2"),
        (282, "Outline Code3"),
        (284, "Outline Code4"),
        (286, "Outline Code5"),
        (288, "Outline Code6"),
        (290, "Outline Code7"),
        (292, "Outline Code8"),
        (294, "Outline Code9"),
        (296, "Outline Code10"),
        (299, "Material Label"),
        (300, "Type"),
        (301, "VAC"),
        (306, "Group By Summary"),
        (311, "Windows User Account"),
        (312, "Hyperlink Screen Tip"),
        (340, "WBS"),
        (442, "Task Outline Number"),
        (686, "Generic"),
        (690, "Request/Demand"),
        (696, "Inactive"),
        (699, "Booking Type"),
        (720, "Actual Work Protected"),
        (721, "Actual Overtime Work Protected"),
        (726, "Created"),
        (737, "Summary"),
        (738, "Error Message"),
        (740, "Default Assignment Owner"),
        (752, "Budget"),
        (753, "Budget Work"),
        (754, "Budget Cost"),
        (755, "Import"),
        (756, "Baseline Budget Work"),
        (757, "Baseline Budget Cost"),
        (800, "Team Assignment Pool"),
        (801, "Cost Center"),
        (803, "Assignment Owner"),
        (808, "Active"),
        (853, "Proposed Start"),
        (856, "Proposed Finish"),
        (859, "Proposed Max Units"),
        (862, "Engagement Status"),
    ],
    units: &[(117,120),(118,121),(119,122),(183,245),(184,246),(185,247),(186,248),(187,249),(188,250),(189,251),(486,496),(487,497),(488,498),(489,499),(490,500),(491,501),(492,502),(493,503),(494,504),(495,505)],
};

pub mod resource {
    pub const ACCRUE_AT: u16 = 19;
    pub const ACTIVE: u16 = 808;
    pub const ACTUAL_COST: u16 = 11;
    pub const ACTUAL_FINISH: u16 = 271;
    pub const ACTUAL_OVERTIME_COST: u16 = 48;
    pub const ACTUAL_OVERTIME_WORK: u16 = 39;
    pub const ACTUAL_OVERTIME_WORK_PROTECTED: u16 = 721;
    pub const ACTUAL_START: u16 = 270;
    pub const ACTUAL_WORK: u16 = 14;
    pub const ACTUAL_WORK_PROTECTED: u16 = 720;
    pub const ACWP: u16 = 53;
    pub const ASSIGNMENT: u16 = 144;
    pub const ASSIGNMENT_DELAY: u16 = 257;
    pub const ASSIGNMENT_OWNER: u16 = 803;
    pub const ASSIGNMENT_UNITS: u16 = 258;
    pub const AVAILABLE_FROM: u16 = 57;
    pub const AVAILABLE_TO: u16 = 58;
    pub const BASELINE_BUDGET_COST: u16 = 757;
    pub const BASELINE_BUDGET_WORK: u16 = 756;
    pub const BASELINE_COST: u16 = 17;
    pub const BASELINE_FINISH: u16 = 260;
    pub const BASELINE_START: u16 = 259;
    pub const BASELINE_WORK: u16 = 15;
    pub const BASE_CALENDAR: u16 = 5;
    pub const BCWP: u16 = 52;
    pub const BCWS: u16 = 51;
    pub const BOOKING_TYPE: u16 = 699;
    pub const BUDGET: u16 = 752;
    pub const BUDGET_COST: u16 = 754;
    pub const BUDGET_WORK: u16 = 753;
    pub const CAN_LEVEL: u16 = 163;
    pub const CODE: u16 = 10;
    pub const CONFIRMED: u16 = 261;
    pub const COST: u16 = 12;
    pub const COST1: u16 = 123;
    pub const COST10: u16 = 172;
    pub const COST2: u16 = 124;
    pub const COST3: u16 = 125;
    pub const COST4: u16 = 166;
    pub const COST5: u16 = 167;
    pub const COST6: u16 = 168;
    pub const COST7: u16 = 169;
    pub const COST8: u16 = 170;
    pub const COST9: u16 = 171;
    pub const COST_CENTER: u16 = 801;
    pub const COST_PER_USE: u16 = 18;
    pub const COST_RATE_TABLE: u16 = 269;
    pub const COST_VARIANCE: u16 = 24;
    pub const CREATED: u16 = 726;
    pub const CV: u16 = 268;
    pub const DATE1: u16 = 173;
    pub const DATE10: u16 = 182;
    pub const DATE2: u16 = 174;
    pub const DATE3: u16 = 175;
    pub const DATE4: u16 = 176;
    pub const DATE5: u16 = 177;
    pub const DATE6: u16 = 178;
    pub const DATE7: u16 = 179;
    pub const DATE8: u16 = 180;
    pub const DATE9: u16 = 181;
    pub const DEFAULT_ASSIGNMENT_OWNER: u16 = 740;
    pub const DURATION1: u16 = 117;
    pub const DURATION10: u16 = 189;
    pub const DURATION10_UNITS: u16 = 251;
    pub const DURATION1_UNITS: u16 = 120;
    pub const DURATION2: u16 = 118;
    pub const DURATION2_UNITS: u16 = 121;
    pub const DURATION3: u16 = 119;
    pub const DURATION3_UNITS: u16 = 122;
    pub const DURATION4: u16 = 183;
    pub const DURATION4_UNITS: u16 = 245;
    pub const DURATION5: u16 = 184;
    pub const DURATION5_UNITS: u16 = 246;
    pub const DURATION6: u16 = 185;
    pub const DURATION6_UNITS: u16 = 247;
    pub const DURATION7: u16 = 186;
    pub const DURATION7_UNITS: u16 = 248;
    pub const DURATION8: u16 = 187;
    pub const DURATION8_UNITS: u16 = 249;
    pub const DURATION9: u16 = 188;
    pub const DURATION9_UNITS: u16 = 250;
    pub const EMAIL_ADDRESS: u16 = 35;
    pub const ENGAGEMENT_STATUS: u16 = 862;
    pub const ERROR_MESSAGE: u16 = 738;
    pub const FINISH: u16 = 262;
    pub const FINISH1: u16 = 107;
    pub const FINISH10: u16 = 194;
    pub const FINISH2: u16 = 108;
    pub const FINISH3: u16 = 109;
    pub const FINISH4: u16 = 110;
    pub const FINISH5: u16 = 111;
    pub const FINISH6: u16 = 190;
    pub const FINISH7: u16 = 191;
    pub const FINISH8: u16 = 192;
    pub const FINISH9: u16 = 193;
    pub const FLAG1: u16 = 127;
    pub const FLAG10: u16 = 126;
    pub const FLAG11: u16 = 195;
    pub const FLAG12: u16 = 196;
    pub const FLAG13: u16 = 197;
    pub const FLAG14: u16 = 198;
    pub const FLAG15: u16 = 199;
    pub const FLAG16: u16 = 200;
    pub const FLAG17: u16 = 201;
    pub const FLAG18: u16 = 202;
    pub const FLAG19: u16 = 203;
    pub const FLAG2: u16 = 128;
    pub const FLAG20: u16 = 204;
    pub const FLAG3: u16 = 129;
    pub const FLAG4: u16 = 130;
    pub const FLAG5: u16 = 131;
    pub const FLAG6: u16 = 132;
    pub const FLAG7: u16 = 133;
    pub const FLAG8: u16 = 134;
    pub const FLAG9: u16 = 135;
    pub const GENERIC: u16 = 686;
    pub const GROUP: u16 = 3;
    pub const GROUP_BY_SUMMARY: u16 = 306;
    pub const HYPERLINK: u16 = 138;
    pub const HYPERLINK_ADDRESS: u16 = 139;
    pub const HYPERLINK_HREF: u16 = 141;
    pub const HYPERLINK_SCREEN_TIP: u16 = 312;
    pub const HYPERLINK_SUBADDRESS: u16 = 140;
    pub const ID: u16 = 0;
    pub const IMPORT: u16 = 755;
    pub const INACTIVE: u16 = 696;
    pub const INDEX: u16 = 253;
    pub const INDICATORS: u16 = 86;
    pub const INITIALS: u16 = 2;
    pub const LEVELING_DELAY: u16 = 263;
    pub const LINKED_FIELDS: u16 = 34;
    pub const MATERIAL_LABEL: u16 = 299;
    pub const MAX_UNITS: u16 = 4;
    pub const NAME: u16 = 1;
    pub const NOTES: u16 = 28;
    pub const NUMBER1: u16 = 112;
    pub const NUMBER10: u16 = 209;
    pub const NUMBER11: u16 = 210;
    pub const NUMBER12: u16 = 211;
    pub const NUMBER13: u16 = 212;
    pub const NUMBER14: u16 = 213;
    pub const NUMBER15: u16 = 214;
    pub const NUMBER16: u16 = 215;
    pub const NUMBER17: u16 = 216;
    pub const NUMBER18: u16 = 217;
    pub const NUMBER19: u16 = 218;
    pub const NUMBER2: u16 = 113;
    pub const NUMBER20: u16 = 219;
    pub const NUMBER3: u16 = 114;
    pub const NUMBER4: u16 = 115;
    pub const NUMBER5: u16 = 116;
    pub const NUMBER6: u16 = 205;
    pub const NUMBER7: u16 = 206;
    pub const NUMBER8: u16 = 207;
    pub const NUMBER9: u16 = 208;
    pub const OBJECTS: u16 = 33;
    pub const OUTLINE_CODE1: u16 = 278;
    pub const OUTLINE_CODE10: u16 = 296;
    pub const OUTLINE_CODE2: u16 = 280;
    pub const OUTLINE_CODE3: u16 = 282;
    pub const OUTLINE_CODE4: u16 = 284;
    pub const OUTLINE_CODE5: u16 = 286;
    pub const OUTLINE_CODE6: u16 = 288;
    pub const OUTLINE_CODE7: u16 = 290;
    pub const OUTLINE_CODE8: u16 = 292;
    pub const OUTLINE_CODE9: u16 = 294;
    pub const OVERALLOCATED: u16 = 25;
    pub const OVERTIME_COST: u16 = 47;
    pub const OVERTIME_RATE: u16 = 7;
    pub const OVERTIME_RATE_UNITS: u16 = 71;
    pub const OVERTIME_WORK: u16 = 16;
    pub const PEAK: u16 = 26;
    pub const PERCENT_WORK_COMPLETE: u16 = 29;
    pub const PHONETICS: u16 = 252;
    pub const PROJECT: u16 = 273;
    pub const PROPOSED_FINISH: u16 = 856;
    pub const PROPOSED_MAX_UNITS: u16 = 859;
    pub const PROPOSED_START: u16 = 853;
    pub const REGULAR_WORK: u16 = 38;
    pub const REMAINING_COST: u16 = 21;
    pub const REMAINING_OVERTIME_COST: u16 = 49;
    pub const REMAINING_OVERTIME_WORK: u16 = 40;
    pub const REMAINING_WORK: u16 = 22;
    pub const REQUEST_DEMAND: u16 = 690;
    pub const RESPONSE_PENDING: u16 = 264;
    pub const STANDARD_RATE: u16 = 6;
    pub const STANDARD_RATE_UNITS: u16 = 70;
    pub const START: u16 = 265;
    pub const START1: u16 = 102;
    pub const START10: u16 = 224;
    pub const START2: u16 = 103;
    pub const START3: u16 = 104;
    pub const START4: u16 = 105;
    pub const START5: u16 = 106;
    pub const START6: u16 = 220;
    pub const START7: u16 = 221;
    pub const START8: u16 = 222;
    pub const START9: u16 = 223;
    pub const SUBPROJECT_RESOURCE_UNIQUE_ID: u16 = 152;
    pub const SUMMARY: u16 = 737;
    pub const SV: u16 = 54;
    pub const TASK_OUTLINE_NUMBER: u16 = 442;
    pub const TASK_SUMMARY_NAME: u16 = 159;
    pub const TEAM_ASSIGNMENT_POOL: u16 = 800;
    pub const TEAM_STATUS_PENDING: u16 = 266;
    pub const TEXT1: u16 = 8;
    pub const TEXT10: u16 = 101;
    pub const TEXT11: u16 = 225;
    pub const TEXT12: u16 = 226;
    pub const TEXT13: u16 = 227;
    pub const TEXT14: u16 = 228;
    pub const TEXT15: u16 = 229;
    pub const TEXT16: u16 = 230;
    pub const TEXT17: u16 = 231;
    pub const TEXT18: u16 = 232;
    pub const TEXT19: u16 = 233;
    pub const TEXT2: u16 = 9;
    pub const TEXT20: u16 = 234;
    pub const TEXT21: u16 = 235;
    pub const TEXT22: u16 = 236;
    pub const TEXT23: u16 = 237;
    pub const TEXT24: u16 = 238;
    pub const TEXT25: u16 = 239;
    pub const TEXT26: u16 = 240;
    pub const TEXT27: u16 = 241;
    pub const TEXT28: u16 = 242;
    pub const TEXT29: u16 = 243;
    pub const TEXT3: u16 = 30;
    pub const TEXT30: u16 = 244;
    pub const TEXT4: u16 = 31;
    pub const TEXT5: u16 = 32;
    pub const TEXT6: u16 = 97;
    pub const TEXT7: u16 = 98;
    pub const TEXT8: u16 = 99;
    pub const TEXT9: u16 = 100;
    pub const TYPE: u16 = 300;
    pub const UNIQUE_ID: u16 = 27;
    pub const UPDATE_NEEDED: u16 = 267;
    pub const VAC: u16 = 301;
    pub const WBS: u16 = 340;
    pub const WINDOWS_USER_ACCOUNT: u16 = 311;
    pub const WORK: u16 = 13;
    pub const WORKGROUP: u16 = 272;
    pub const WORK_CONTOUR: u16 = 164;
    pub const WORK_VARIANCE: u16 = 23;
}

pub static ASSIGNMENT: FieldTable = FieldTable {
    base: 0x0F400000,
    dt: &[9,9,9,9,9,1,1,12,8,8,8,8,8,8,8,0,8,0,0,12,2,2,2,2,2,20,4,4,4,4,4,4,4,0,4,4,4,4,4,27,0,0,0,11,1,0,0,0,0,27,27,27,27,27,0,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,13,5,5,5,5,0,0,0,0,10,0,0,0,0,0,0,27,1,1,1,1,1,1,1,1,1,1,2,2,2,2,2,2,2,2,2,2,3,3,3,3,3,6,6,6,7,7,7,4,4,4,5,5,5,5,5,5,5,5,5,5,5,0,0,5,0,0,0,0,0,0,1,0,27,6,2,2,0,0,27,0,1,1,1,1,0,0,0,4,4,4,4,4,4,4,2,2,2,2,2,2,2,2,2,2,6,6,6,6,6,6,6,2,2,2,2,2,5,5,5,5,5,5,5,5,5,5,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,2,2,2,2,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,7,7,7,7,7,7,7,0,9,4,8,0,0,0,0,0,0,0,0,0,0,0,0,0,6,6,2,0,0,0,0,0,21,4,0,0,0,1,27,0,0,1,0,0,0,0,0,0,1,0,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,8,4,27,27,0,0,2,2,0,1,0,4,4,4,4,4,4,4,4,4,4,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,6,6,6,6,6,6,6,6,6,6,0,0,0,0,0,0,0,0,0,0,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,25,0,1,0,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,8,8,0,0,2,0,23,23,23,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27,8,4,27,27],
    canon: &[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,181,182,183,184,185,186,187,188,189,190,191,192,193,194,195,196,197,198,199,200,201,202,203,204,205,206,207,208,209,210,211,212,213,214,215,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230,231,232,233,234,235,236,237,238,239,240,241,242,243,244,245,246,247,248,249,250,251,252,253,254,255,256,257,258,259,260,261,262,263,264,265,266,267,268,269,270,271,272,273,274,275,276,277,278,279,280,281,282,283,284,285,286,287,288,289,290,291,292,293,294,295,296,297,298,299,300,301,302,303,304,305,306,307,308,309,310,311,312,313,314,315,316,317,318,319,320,321,322,323,324,325,326,327,328,329,330,331,332,333,334,335,336,337,338,339,340,341,342,343,344,345,346,347,348,349,350,351,352,353,354,355,356,357,358,359,360,361,362,363,364,365,366,367,368,369,370,371,372,373,374,375,376,377,378,379,380,381,382,383,384,385,386,387,388,389,390,391,392,393,394,395,396,397,398,399,400,401,402,403,404,405,406,407,408,409,410,411,412,413,414,415,416,417,418,419,420,421,422,423,424,425,426,427,428,429,430,431,432,433,434,435,436,437,438,439,440,441,442,443,444,445,446,447,448,449,450,451,452,453,454,455,456,457,458,459,460,461,462,463,464,465,466,467,468,469,470,471,472,473,474,475,476,477,478,479,480,481,482,483,484,485,486,487,488,489,490,491,492,493,494,495,496,497,498,499,500,501,502,503,504,505,506,507,508,509,510,511,512,513,514,515,516,517,518,519,520,521,522,523,524,525,526,527,528,529,530,531,532,533,534,535,536,537,538,539,540,541,542,543,544,545,546,547,548,549,550,551,552,553,554,555,556,557,558,559,560,561,562,563,564,565,566,567,568,569,570,571,572,573,574,575,576,577,578,579,580,581,582,583,584,585,586,587,588,589,590,591,592,593,594,595,596,597,598,599,600,601,602,603,604,605,606,607,608,609,610,611,612,613,614,615,616,617,618,619,620,621,622,623,624,625,626,627,628,629,630,631,632,633,634,635,636,637,638,639,640,641,642,643,644,645,646,647,648,649,650,651,652,653,654,655,656,657,658,659,660,661,662,663,664,665,666,667,668,669,670,671,672,673,674,675,676,677,678,679,680,681,682,683,684,685,686,687,688,689,690,691,692,693,694,695,696,697,698,699,700,701,702,703,704,705,706,707,708,709,710,711,712,713,714,715,716],
    names: &[
        (0, "Unique ID"),
        (1, "Task Unique ID"),
        (2, "Resource Unique ID"),
        (3, "Task ID"),
        (4, "Resource ID"),
        (5, "Task Name"),
        (6, "Resource Name"),
        (7, "Assignment Units"),
        (8, "Work"),
        (9, "Overtime Work"),
        (10, "Actual Work"),
        (11, "Regular Work"),
        (12, "Remaining Work"),
        (13, "Actual Overtime Work"),
        (14, "Remaining Overtime Work"),
        (16, "Baseline Work"),
        (19, "Peak"),
        (20, "Start"),
        (21, "Finish"),
        (22, "Actual Start"),
        (23, "Actual Finish"),
        (24, "Resume"),
        (25, "Assignment Delay"),
        (26, "Cost"),
        (27, "Overtime Cost"),
        (28, "Actual Cost"),
        (29, "Remaining Cost"),
        (30, "Actual Overtime Cost"),
        (31, "Remaining Overtime Cost"),
        (32, "Baseline Cost"),
        (34, "BCWS"),
        (35, "BCWP"),
        (36, "ACWP"),
        (37, "SV"),
        (38, "Cost Variance"),
        (39, "Work Contour"),
        (43, "Percent Work Complete"),
        (44, "Project"),
        (55, "Leveling Delay Units"),
        (71, "Notes"),
        (72, "Confirmed"),
        (73, "Response Pending"),
        (74, "Update Needed"),
        (75, "Team Status Pending"),
        (80, "Cost Rate Table"),
        (88, "Text1"),
        (89, "Text2"),
        (90, "Text3"),
        (91, "Text4"),
        (92, "Text5"),
        (93, "Text6"),
        (94, "Text7"),
        (95, "Text8"),
        (96, "Text9"),
        (97, "Text10"),
        (98, "Start1"),
        (99, "Start2"),
        (100, "Start3"),
        (101, "Start4"),
        (102, "Start5"),
        (103, "Finish1"),
        (104, "Finish2"),
        (105, "Finish3"),
        (106, "Finish4"),
        (107, "Finish5"),
        (108, "Number1"),
        (109, "Number2"),
        (110, "Number3"),
        (111, "Number4"),
        (112, "Number5"),
        (113, "Duration1"),
        (114, "Duration2"),
        (115, "Duration3"),
        (116, "Duration1 Units"),
        (117, "Duration2 Units"),
        (118, "Duration3 Units"),
        (119, "Cost1"),
        (120, "Cost2"),
        (121, "Cost3"),
        (122, "Flag10"),
        (123, "Flag1"),
        (124, "Flag2"),
        (125, "Flag3"),
        (126, "Flag4"),
        (127, "Flag5"),
        (128, "Flag6"),
        (129, "Flag7"),
        (130, "Flag8"),
        (131, "Flag9"),
        (132, "Linked Fields"),
        (135, "Overallocated"),
        (142, "Task Summary Name"),
        (145, "Leveling Delay"),
        (146, "Baseline Start"),
        (147, "Baseline Finish"),
        (152, "Hyperlink"),
        (153, "Hyperlink Address"),
        (154, "Hyperlink Subaddress"),
        (155, "Hyperlink Href"),
        (159, "Cost4"),
        (160, "Cost5"),
        (161, "Cost6"),
        (162, "Cost7"),
        (163, "Cost8"),
        (164, "Cost9"),
        (165, "Cost10"),
        (166, "Date1"),
        (167, "Date2"),
        (168, "Date3"),
        (169, "Date4"),
        (170, "Date5"),
        (171, "Date6"),
        (172, "Date7"),
        (173, "Date8"),
        (174, "Date9"),
        (175, "Date10"),
        (176, "Duration4"),
        (177, "Duration5"),
        (178, "Duration6"),
        (179, "Duration7"),
        (180, "Duration8"),
        (181, "Duration9"),
        (182, "Duration10"),
        (183, "Finish6"),
        (184, "Finish7"),
        (185, "Finish8"),
        (186, "Finish9"),
        (187, "Finish10"),
        (188, "Flag11"),
        (189, "Flag12"),
        (190, "Flag13"),
        (191, "Flag14"),
        (192, "Flag15"),
        (193, "Flag16"),
        (194, "Flag17"),
        (195, "Flag18"),
        (196, "Flag19"),
        (197, "Flag20"),
        (198, "Number6"),
        (199, "Number7"),
        (200, "Number8"),
        (201, "Number9"),
        (202, "Number10"),
        (203, "Number11"),
        (204, "Number12"),
        (205, "Number13"),
        (206, "Number14"),
        (207, "Number15"),
        (208, "Number16"),
        (209, "Number17"),
        (210, "Number18"),
        (211, "Number19"),
        (212, "Number20"),
        (213, "Start6"),
        (214, "Start7"),
        (215, "Start8"),
        (216, "Start9"),
        (217, "Start10"),
        (218, "Text11"),
        (219, "Text12"),
        (220, "Text13"),
        (221, "Text14"),
        (222, "Text15"),
        (223, "Text16"),
        (224, "Text17"),
        (225, "Text18"),
        (226, "Text19"),
        (227, "Text20"),
        (228, "Text21"),
        (229, "Text22"),
        (230, "Text23"),
        (231, "Text24"),
        (232, "Text25"),
        (233, "Text26"),
        (234, "Text27"),
        (235, "Text28"),
        (236, "Text29"),
        (237, "Text30"),
        (238, "Duration4 Units"),
        (239, "Duration5 Units"),
        (240, "Duration6 Units"),
        (241, "Duration7 Units"),
        (242, "Duration8 Units"),
        (243, "Duration9 Units"),
        (244, "Duration10 Units"),
        (246, "Index"),
        (247, "CV"),
        (248, "Work Variance"),
        (262, "Start Variance"),
        (263, "Finish Variance"),
        (264, "Stop"),
        (270, "Variable Rate Units"),
        (271, "VAC"),
        (275, "Fixed Material Assignment"),
        (276, "Resource Type"),
        (279, "Hyperlink Screen Tip"),
        (286, "WBS"),
        (379, "Task Outline Number"),
        (605, "Resource Request Type"),
        (630, "Actual Work Protected"),
        (631, "Actual Overtime Work Protected"),
        (634, "Created"),
        (646, "Summary"),
        (668, "Owner"),
        (669, "Budget Work"),
        (670, "Budget Cost"),
        (673, "Baseline Budget Work"),
        (674, "Baseline Budget Cost"),
    ],
    units: &[(113,116),(114,117),(115,118),(145,55),(176,238),(177,239),(178,240),(179,241),(180,242),(181,243),(182,244)],
};

pub mod assignment {
    pub const ACTUAL_COST: u16 = 28;
    pub const ACTUAL_FINISH: u16 = 23;
    pub const ACTUAL_OVERTIME_COST: u16 = 30;
    pub const ACTUAL_OVERTIME_WORK: u16 = 13;
    pub const ACTUAL_OVERTIME_WORK_PROTECTED: u16 = 631;
    pub const ACTUAL_START: u16 = 22;
    pub const ACTUAL_WORK: u16 = 10;
    pub const ACTUAL_WORK_PROTECTED: u16 = 630;
    pub const ACWP: u16 = 36;
    pub const ASSIGNMENT_DELAY: u16 = 25;
    pub const ASSIGNMENT_UNITS: u16 = 7;
    pub const BASELINE_BUDGET_COST: u16 = 674;
    pub const BASELINE_BUDGET_WORK: u16 = 673;
    pub const BASELINE_COST: u16 = 32;
    pub const BASELINE_FINISH: u16 = 147;
    pub const BASELINE_START: u16 = 146;
    pub const BASELINE_WORK: u16 = 16;
    pub const BCWP: u16 = 35;
    pub const BCWS: u16 = 34;
    pub const BUDGET_COST: u16 = 670;
    pub const BUDGET_WORK: u16 = 669;
    pub const CONFIRMED: u16 = 72;
    pub const COST: u16 = 26;
    pub const COST1: u16 = 119;
    pub const COST10: u16 = 165;
    pub const COST2: u16 = 120;
    pub const COST3: u16 = 121;
    pub const COST4: u16 = 159;
    pub const COST5: u16 = 160;
    pub const COST6: u16 = 161;
    pub const COST7: u16 = 162;
    pub const COST8: u16 = 163;
    pub const COST9: u16 = 164;
    pub const COST_RATE_TABLE: u16 = 80;
    pub const COST_VARIANCE: u16 = 38;
    pub const CREATED: u16 = 634;
    pub const CV: u16 = 247;
    pub const DATE1: u16 = 166;
    pub const DATE10: u16 = 175;
    pub const DATE2: u16 = 167;
    pub const DATE3: u16 = 168;
    pub const DATE4: u16 = 169;
    pub const DATE5: u16 = 170;
    pub const DATE6: u16 = 171;
    pub const DATE7: u16 = 172;
    pub const DATE8: u16 = 173;
    pub const DATE9: u16 = 174;
    pub const DURATION1: u16 = 113;
    pub const DURATION10: u16 = 182;
    pub const DURATION10_UNITS: u16 = 244;
    pub const DURATION1_UNITS: u16 = 116;
    pub const DURATION2: u16 = 114;
    pub const DURATION2_UNITS: u16 = 117;
    pub const DURATION3: u16 = 115;
    pub const DURATION3_UNITS: u16 = 118;
    pub const DURATION4: u16 = 176;
    pub const DURATION4_UNITS: u16 = 238;
    pub const DURATION5: u16 = 177;
    pub const DURATION5_UNITS: u16 = 239;
    pub const DURATION6: u16 = 178;
    pub const DURATION6_UNITS: u16 = 240;
    pub const DURATION7: u16 = 179;
    pub const DURATION7_UNITS: u16 = 241;
    pub const DURATION8: u16 = 180;
    pub const DURATION8_UNITS: u16 = 242;
    pub const DURATION9: u16 = 181;
    pub const DURATION9_UNITS: u16 = 243;
    pub const FINISH: u16 = 21;
    pub const FINISH1: u16 = 103;
    pub const FINISH10: u16 = 187;
    pub const FINISH2: u16 = 104;
    pub const FINISH3: u16 = 105;
    pub const FINISH4: u16 = 106;
    pub const FINISH5: u16 = 107;
    pub const FINISH6: u16 = 183;
    pub const FINISH7: u16 = 184;
    pub const FINISH8: u16 = 185;
    pub const FINISH9: u16 = 186;
    pub const FINISH_VARIANCE: u16 = 263;
    pub const FIXED_MATERIAL_ASSIGNMENT: u16 = 275;
    pub const FLAG1: u16 = 123;
    pub const FLAG10: u16 = 122;
    pub const FLAG11: u16 = 188;
    pub const FLAG12: u16 = 189;
    pub const FLAG13: u16 = 190;
    pub const FLAG14: u16 = 191;
    pub const FLAG15: u16 = 192;
    pub const FLAG16: u16 = 193;
    pub const FLAG17: u16 = 194;
    pub const FLAG18: u16 = 195;
    pub const FLAG19: u16 = 196;
    pub const FLAG2: u16 = 124;
    pub const FLAG20: u16 = 197;
    pub const FLAG3: u16 = 125;
    pub const FLAG4: u16 = 126;
    pub const FLAG5: u16 = 127;
    pub const FLAG6: u16 = 128;
    pub const FLAG7: u16 = 129;
    pub const FLAG8: u16 = 130;
    pub const FLAG9: u16 = 131;
    pub const HYPERLINK: u16 = 152;
    pub const HYPERLINK_ADDRESS: u16 = 153;
    pub const HYPERLINK_HREF: u16 = 155;
    pub const HYPERLINK_SCREEN_TIP: u16 = 279;
    pub const HYPERLINK_SUBADDRESS: u16 = 154;
    pub const INDEX: u16 = 246;
    pub const LEVELING_DELAY: u16 = 145;
    pub const LEVELING_DELAY_UNITS: u16 = 55;
    pub const LINKED_FIELDS: u16 = 132;
    pub const NOTES: u16 = 71;
    pub const NUMBER1: u16 = 108;
    pub const NUMBER10: u16 = 202;
    pub const NUMBER11: u16 = 203;
    pub const NUMBER12: u16 = 204;
    pub const NUMBER13: u16 = 205;
    pub const NUMBER14: u16 = 206;
    pub const NUMBER15: u16 = 207;
    pub const NUMBER16: u16 = 208;
    pub const NUMBER17: u16 = 209;
    pub const NUMBER18: u16 = 210;
    pub const NUMBER19: u16 = 211;
    pub const NUMBER2: u16 = 109;
    pub const NUMBER20: u16 = 212;
    pub const NUMBER3: u16 = 110;
    pub const NUMBER4: u16 = 111;
    pub const NUMBER5: u16 = 112;
    pub const NUMBER6: u16 = 198;
    pub const NUMBER7: u16 = 199;
    pub const NUMBER8: u16 = 200;
    pub const NUMBER9: u16 = 201;
    pub const OVERALLOCATED: u16 = 135;
    pub const OVERTIME_COST: u16 = 27;
    pub const OVERTIME_WORK: u16 = 9;
    pub const OWNER: u16 = 668;
    pub const PEAK: u16 = 19;
    pub const PERCENT_WORK_COMPLETE: u16 = 43;
    pub const PROJECT: u16 = 44;
    pub const REGULAR_WORK: u16 = 11;
    pub const REMAINING_COST: u16 = 29;
    pub const REMAINING_OVERTIME_COST: u16 = 31;
    pub const REMAINING_OVERTIME_WORK: u16 = 14;
    pub const REMAINING_WORK: u16 = 12;
    pub const RESOURCE_ID: u16 = 4;
    pub const RESOURCE_NAME: u16 = 6;
    pub const RESOURCE_REQUEST_TYPE: u16 = 605;
    pub const RESOURCE_TYPE: u16 = 276;
    pub const RESOURCE_UNIQUE_ID: u16 = 2;
    pub const RESPONSE_PENDING: u16 = 73;
    pub const RESUME: u16 = 24;
    pub const START: u16 = 20;
    pub const START1: u16 = 98;
    pub const START10: u16 = 217;
    pub const START2: u16 = 99;
    pub const START3: u16 = 100;
    pub const START4: u16 = 101;
    pub const START5: u16 = 102;
    pub const START6: u16 = 213;
    pub const START7: u16 = 214;
    pub const START8: u16 = 215;
    pub const START9: u16 = 216;
    pub const START_VARIANCE: u16 = 262;
    pub const STOP: u16 = 264;
    pub const SUMMARY: u16 = 646;
    pub const SV: u16 = 37;
    pub const TASK_ID: u16 = 3;
    pub const TASK_NAME: u16 = 5;
    pub const TASK_OUTLINE_NUMBER: u16 = 379;
    pub const TASK_SUMMARY_NAME: u16 = 142;
    pub const TASK_UNIQUE_ID: u16 = 1;
    pub const TEAM_STATUS_PENDING: u16 = 75;
    pub const TEXT1: u16 = 88;
    pub const TEXT10: u16 = 97;
    pub const TEXT11: u16 = 218;
    pub const TEXT12: u16 = 219;
    pub const TEXT13: u16 = 220;
    pub const TEXT14: u16 = 221;
    pub const TEXT15: u16 = 222;
    pub const TEXT16: u16 = 223;
    pub const TEXT17: u16 = 224;
    pub const TEXT18: u16 = 225;
    pub const TEXT19: u16 = 226;
    pub const TEXT2: u16 = 89;
    pub const TEXT20: u16 = 227;
    pub const TEXT21: u16 = 228;
    pub const TEXT22: u16 = 229;
    pub const TEXT23: u16 = 230;
    pub const TEXT24: u16 = 231;
    pub const TEXT25: u16 = 232;
    pub const TEXT26: u16 = 233;
    pub const TEXT27: u16 = 234;
    pub const TEXT28: u16 = 235;
    pub const TEXT29: u16 = 236;
    pub const TEXT3: u16 = 90;
    pub const TEXT30: u16 = 237;
    pub const TEXT4: u16 = 91;
    pub const TEXT5: u16 = 92;
    pub const TEXT6: u16 = 93;
    pub const TEXT7: u16 = 94;
    pub const TEXT8: u16 = 95;
    pub const TEXT9: u16 = 96;
    pub const UNIQUE_ID: u16 = 0;
    pub const UPDATE_NEEDED: u16 = 74;
    pub const VAC: u16 = 271;
    pub const VARIABLE_RATE_UNITS: u16 = 270;
    pub const WBS: u16 = 286;
    pub const WORK: u16 = 8;
    pub const WORK_CONTOUR: u16 = 39;
    pub const WORK_VARIANCE: u16 = 248;
}
